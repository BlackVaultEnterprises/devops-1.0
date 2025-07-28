import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 10 },  // Ramp up to 10 users
    { duration: '5m', target: 10 },  // Stay at 10 users
    { duration: '2m', target: 50 },  // Ramp up to 50 users
    { duration: '5m', target: 50 },  // Stay at 50 users
    { duration: '2m', target: 100 }, // Ramp up to 100 users
    { duration: '5m', target: 100 }, // Stay at 100 users
    { duration: '2m', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
    http_req_failed: ['rate<0.1'],    // Error rate must be below 10%
    errors: ['rate<0.1'],
  },
};

// Test data
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

// Main test function
export default function () {
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'User-Agent': 'K6-Load-Test',
    },
  };

  // Test scenarios
  const responses = {
    health: http.get(`${BASE_URL}/health`, params),
    apiStatus: http.get(`${BASE_URL}/api/status`, params),
    metrics: http.get(`${BASE_URL}/metrics`, params),
  };

  // Health check
  check(responses.health, {
    'health check is 200': (r) => r.status === 200,
    'health check response time < 200ms': (r) => r.timings.duration < 200,
  });

  // API status check
  check(responses.apiStatus, {
    'api status is 200': (r) => r.status === 200,
    'api status response time < 500ms': (r) => r.timings.duration < 500,
  });

  // Metrics endpoint check
  check(responses.metrics, {
    'metrics endpoint is 200': (r) => r.status === 200,
    'metrics response time < 1000ms': (r) => r.timings.duration < 1000,
  });

  // Record errors
  errorRate.add(
    responses.health.status !== 200 ||
    responses.apiStatus.status !== 200 ||
    responses.metrics.status !== 200
  );

  // Think time between requests
  sleep(1);
}

// Setup function (runs once at the beginning)
export function setup() {
  console.log('Starting load test against:', BASE_URL);
  
  // Verify the application is accessible
  const healthCheck = http.get(`${BASE_URL}/health`);
  check(healthCheck, {
    'application is accessible': (r) => r.status === 200,
  });
  
  if (healthCheck.status !== 200) {
    throw new Error('Application is not accessible');
  }
}

// Teardown function (runs once at the end)
export function teardown(data) {
  console.log('Load test completed');
}

// Handle summary
export function handleSummary(data) {
  return {
    'performance-results/summary.json': JSON.stringify(data),
    stdout: textSummary(data, { indent: ' ', enableColors: true }),
  };
} 