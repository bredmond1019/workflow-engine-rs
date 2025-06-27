// Visual Testing Dashboard JavaScript

// Global state
let dashboardState = {
    services: {},
    testResults: [],
    alerts: [],
    metrics: {
        responseTime: [],
        successRate: [],
        errorDistribution: {},
        testCoverage: {}
    },
    charts: {},
    network: null,
    refreshInterval: null,
    config: null
};

// Initialize dashboard
document.addEventListener('DOMContentLoaded', async () => {
    await loadConfiguration();
    initializeCharts();
    initializeTopology();
    setupEventListeners();
    startAutoRefresh();
    await refreshDashboard();
});

// Load configuration
async function loadConfiguration() {
    try {
        const response = await fetch('../test-config.yml');
        const yamlText = await response.text();
        // Simple YAML parsing (in production, use a proper YAML parser)
        dashboardState.config = parseSimpleYAML(yamlText);
    } catch (error) {
        console.error('Failed to load configuration:', error);
        dashboardState.config = getDefaultConfig();
    }
}

// Initialize charts
function initializeCharts() {
    // Response Time Chart
    const responseTimeCtx = document.getElementById('response-time-chart').getContext('2d');
    dashboardState.charts.responseTime = new Chart(responseTimeCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: []
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: 'Response Time (ms)'
                    }
                }
            },
            plugins: {
                legend: {
                    display: true,
                    position: 'bottom'
                }
            }
        }
    });

    // Success Rate Chart
    const successRateCtx = document.getElementById('success-rate-chart').getContext('2d');
    dashboardState.charts.successRate = new Chart(successRateCtx, {
        type: 'bar',
        data: {
            labels: [],
            datasets: [{
                label: 'Success Rate',
                data: [],
                backgroundColor: 'rgba(75, 192, 192, 0.6)',
                borderColor: 'rgba(75, 192, 192, 1)',
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    beginAtZero: true,
                    max: 100,
                    title: {
                        display: true,
                        text: 'Success Rate (%)'
                    }
                }
            }
        }
    });

    // Error Distribution Chart
    const errorDistCtx = document.getElementById('error-distribution-chart').getContext('2d');
    dashboardState.charts.errorDistribution = new Chart(errorDistCtx, {
        type: 'doughnut',
        data: {
            labels: [],
            datasets: [{
                data: [],
                backgroundColor: [
                    'rgba(255, 99, 132, 0.6)',
                    'rgba(54, 162, 235, 0.6)',
                    'rgba(255, 206, 86, 0.6)',
                    'rgba(75, 192, 192, 0.6)',
                    'rgba(153, 102, 255, 0.6)'
                ]
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'right'
                }
            }
        }
    });

    // Test Coverage Chart
    const coverageCtx = document.getElementById('test-coverage-chart').getContext('2d');
    dashboardState.charts.testCoverage = new Chart(coverageCtx, {
        type: 'radar',
        data: {
            labels: ['Health', 'GraphQL', 'Federation', 'Performance', 'Integration'],
            datasets: [{
                label: 'Coverage',
                data: [0, 0, 0, 0, 0],
                backgroundColor: 'rgba(54, 162, 235, 0.2)',
                borderColor: 'rgba(54, 162, 235, 1)',
                pointBackgroundColor: 'rgba(54, 162, 235, 1)',
                pointBorderColor: '#fff',
                pointHoverBackgroundColor: '#fff',
                pointHoverBorderColor: 'rgba(54, 162, 235, 1)'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                r: {
                    beginAtZero: true,
                    max: 100
                }
            }
        }
    });
}

// Initialize network topology
function initializeTopology() {
    const container = document.getElementById('network-topology');
    
    const nodes = new vis.DataSet([]);
    const edges = new vis.DataSet([]);
    
    const options = {
        nodes: {
            shape: 'box',
            margin: 10,
            font: {
                size: 14,
                color: '#ffffff'
            }
        },
        edges: {
            arrows: 'to',
            smooth: {
                type: 'cubicBezier',
                forceDirection: 'horizontal',
                roundness: 0.4
            }
        },
        layout: {
            hierarchical: {
                direction: 'LR',
                sortMethod: 'directed',
                levelSeparation: 200,
                nodeSpacing: 100
            }
        },
        physics: {
            enabled: false
        }
    };
    
    dashboardState.network = new vis.Network(container, { nodes, edges }, options);
}

// Setup event listeners
function setupEventListeners() {
    // Run all tests button
    document.getElementById('run-all-tests').addEventListener('click', runAllTests);
    
    // Refresh dashboard button
    document.getElementById('refresh-dashboard').addEventListener('click', refreshDashboard);
    
    // Auto-refresh toggle
    document.getElementById('auto-refresh').addEventListener('change', (e) => {
        if (e.target.checked) {
            startAutoRefresh();
        } else {
            stopAutoRefresh();
        }
    });
    
    // Refresh interval change
    document.getElementById('refresh-interval').addEventListener('change', (e) => {
        if (document.getElementById('auto-refresh').checked) {
            stopAutoRefresh();
            startAutoRefresh();
        }
    });
    
    // Test filters
    document.getElementById('test-category-filter').addEventListener('change', filterTestResults);
    document.getElementById('test-status-filter').addEventListener('change', filterTestResults);
    document.getElementById('test-search').addEventListener('input', filterTestResults);
}

// Start auto-refresh
function startAutoRefresh() {
    const interval = parseInt(document.getElementById('refresh-interval').value);
    dashboardState.refreshInterval = setInterval(refreshDashboard, interval);
}

// Stop auto-refresh
function stopAutoRefresh() {
    if (dashboardState.refreshInterval) {
        clearInterval(dashboardState.refreshInterval);
        dashboardState.refreshInterval = null;
    }
}

// Refresh dashboard
async function refreshDashboard() {
    try {
        await updateServiceStatus();
        await updateTestResults();
        updateMetrics();
        updateTopology();
        updateFooterStats();
        document.getElementById('last-update').textContent = new Date().toLocaleTimeString();
    } catch (error) {
        console.error('Dashboard refresh failed:', error);
        showAlert('error', 'Failed to refresh dashboard: ' + error.message);
    }
}

// Update service status
async function updateServiceStatus() {
    const serviceGrid = document.getElementById('service-grid');
    serviceGrid.innerHTML = '';
    
    const services = dashboardState.config?.services || {};
    
    for (const [serviceId, service] of Object.entries(services)) {
        const status = await checkServiceHealth(service);
        dashboardState.services[serviceId] = { ...service, status };
        
        const serviceCard = createServiceCard(serviceId, service, status);
        serviceGrid.appendChild(serviceCard);
    }
    
    updateOverallStatus();
}

// Check service health
async function checkServiceHealth(service) {
    try {
        const response = await fetch(service.url + service.health_endpoint, {
            method: 'GET',
            signal: AbortSignal.timeout(service.timeout || 5000)
        });
        
        if (response.ok) {
            const data = await response.json();
            return {
                status: 'healthy',
                responseTime: Date.now() - startTime,
                details: data
            };
        } else {
            return {
                status: 'error',
                error: `HTTP ${response.status}`,
                responseTime: Date.now() - startTime
            };
        }
    } catch (error) {
        return {
            status: 'error',
            error: error.message,
            responseTime: null
        };
    }
}

// Create service card
function createServiceCard(serviceId, service, status) {
    const card = document.createElement('div');
    card.className = `service-card ${status.status}`;
    card.onclick = () => showServiceDetail(serviceId);
    
    card.innerHTML = `
        <div class="service-header">
            <h3>${service.name}</h3>
            <span class="service-category">${service.category}</span>
        </div>
        <div class="service-status">
            <span class="status-indicator ${status.status}"></span>
            <span class="status-text">${status.status}</span>
        </div>
        <div class="service-metrics">
            <div class="metric">
                <span class="metric-label">Response Time</span>
                <span class="metric-value">${status.responseTime ? status.responseTime + 'ms' : 'N/A'}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Endpoint</span>
                <span class="metric-value">${service.url}</span>
            </div>
        </div>
        ${status.error ? `<div class="service-error">${status.error}</div>` : ''}
    `;
    
    return card;
}

// Update test results
async function updateTestResults() {
    // In a real implementation, this would fetch from the test runner
    // For now, we'll simulate some test results
    const newResults = await fetchTestResults();
    dashboardState.testResults = [...newResults, ...dashboardState.testResults].slice(0, 100);
    
    renderTestResults();
}

// Fetch test results (simulated)
async function fetchTestResults() {
    // This would normally fetch from the test runner API
    // Simulating test results for demonstration
    const results = [];
    const categories = ['health', 'graphql', 'federation', 'performance', 'integration'];
    const statuses = ['passed', 'failed', 'warning'];
    
    for (let i = 0; i < 5; i++) {
        results.push({
            id: Date.now() + i,
            timestamp: new Date(),
            service: Object.keys(dashboardState.services)[Math.floor(Math.random() * Object.keys(dashboardState.services).length)],
            test: `Test ${Math.floor(Math.random() * 100)}`,
            category: categories[Math.floor(Math.random() * categories.length)],
            status: statuses[Math.floor(Math.random() * statuses.length)],
            duration: Math.floor(Math.random() * 1000) + 100,
            details: {
                message: 'Test completed successfully',
                assertions: Math.floor(Math.random() * 10) + 1
            }
        });
    }
    
    return results;
}

// Render test results
function renderTestResults() {
    const tbody = document.querySelector('#test-results tbody');
    tbody.innerHTML = '';
    
    const filteredResults = filterTestResultsData();
    
    filteredResults.forEach(result => {
        const row = document.createElement('tr');
        row.className = `test-result ${result.status}`;
        row.onclick = () => showTestDetail(result);
        
        row.innerHTML = `
            <td>${result.timestamp.toLocaleTimeString()}</td>
            <td>${dashboardState.services[result.service]?.name || result.service}</td>
            <td>${result.test}</td>
            <td><span class="category-badge ${result.category}">${result.category}</span></td>
            <td><span class="status-badge ${result.status}">${result.status}</span></td>
            <td>${result.duration}ms</td>
            <td><button class="btn-small" onclick="event.stopPropagation(); showTestDetail(${JSON.stringify(result).replace(/"/g, '&quot;')})">Details</button></td>
        `;
        
        tbody.appendChild(row);
    });
}

// Filter test results data
function filterTestResultsData() {
    const categoryFilter = document.getElementById('test-category-filter').value;
    const statusFilter = document.getElementById('test-status-filter').value;
    const searchTerm = document.getElementById('test-search').value.toLowerCase();
    
    return dashboardState.testResults.filter(result => {
        if (categoryFilter !== 'all' && result.category !== categoryFilter) return false;
        if (statusFilter !== 'all' && result.status !== statusFilter) return false;
        if (searchTerm && !result.test.toLowerCase().includes(searchTerm) && 
            !dashboardState.services[result.service]?.name.toLowerCase().includes(searchTerm)) return false;
        return true;
    });
}

// Filter test results (event handler)
function filterTestResults() {
    renderTestResults();
}

// Update metrics
function updateMetrics() {
    updateResponseTimeChart();
    updateSuccessRateChart();
    updateErrorDistributionChart();
    updateTestCoverageChart();
}

// Update response time chart
function updateResponseTimeChart() {
    const chart = dashboardState.charts.responseTime;
    const services = Object.entries(dashboardState.services);
    
    // Update labels (time)
    const now = new Date().toLocaleTimeString();
    if (chart.data.labels.length > 20) {
        chart.data.labels.shift();
    }
    chart.data.labels.push(now);
    
    // Update datasets
    services.forEach(([serviceId, service], index) => {
        if (!chart.data.datasets[index]) {
            chart.data.datasets.push({
                label: service.name,
                data: [],
                borderColor: getServiceColor(index),
                backgroundColor: getServiceColor(index, 0.2),
                tension: 0.4
            });
        }
        
        const dataset = chart.data.datasets[index];
        if (dataset.data.length > 20) {
            dataset.data.shift();
        }
        dataset.data.push(service.status?.responseTime || 0);
    });
    
    chart.update();
}

// Update success rate chart
function updateSuccessRateChart() {
    const chart = dashboardState.charts.successRate;
    const services = Object.entries(dashboardState.services);
    
    chart.data.labels = services.map(([_, service]) => service.name);
    chart.data.datasets[0].data = services.map(([_, service]) => {
        // Calculate success rate based on recent test results
        const serviceTests = dashboardState.testResults.filter(r => r.service === _);
        const passed = serviceTests.filter(r => r.status === 'passed').length;
        return serviceTests.length > 0 ? (passed / serviceTests.length) * 100 : 0;
    });
    
    chart.update();
}

// Update error distribution chart
function updateErrorDistributionChart() {
    const chart = dashboardState.charts.errorDistribution;
    const errorTypes = {};
    
    dashboardState.testResults
        .filter(r => r.status === 'failed')
        .forEach(result => {
            const errorType = result.details?.errorType || 'Unknown';
            errorTypes[errorType] = (errorTypes[errorType] || 0) + 1;
        });
    
    chart.data.labels = Object.keys(errorTypes);
    chart.data.datasets[0].data = Object.values(errorTypes);
    chart.update();
}

// Update test coverage chart
function updateTestCoverageChart() {
    const chart = dashboardState.charts.testCoverage;
    const categories = ['health', 'graphql', 'federation', 'performance', 'integration'];
    
    const coverage = categories.map(category => {
        const categoryTests = dashboardState.testResults.filter(r => r.category === category);
        const passed = categoryTests.filter(r => r.status === 'passed').length;
        return categoryTests.length > 0 ? (passed / categoryTests.length) * 100 : 0;
    });
    
    chart.data.datasets[0].data = coverage;
    chart.update();
}

// Update network topology
function updateTopology() {
    const nodes = [];
    const edges = [];
    
    // Add gateway node
    nodes.push({
        id: 'gateway',
        label: 'GraphQL Gateway',
        level: 0,
        color: getStatusColor('healthy'),
        font: { color: '#ffffff' }
    });
    
    // Add service nodes
    Object.entries(dashboardState.services).forEach(([serviceId, service], index) => {
        const level = service.category === 'core' ? 1 : 2;
        nodes.push({
            id: serviceId,
            label: service.name,
            level: level,
            color: getStatusColor(service.status?.status || 'unknown'),
            font: { color: '#ffffff' }
        });
        
        // Add edges based on service relationships
        if (service.category === 'core') {
            edges.push({
                from: 'gateway',
                to: serviceId,
                color: { color: '#666666' }
            });
        } else if (service.category === 'microservice') {
            edges.push({
                from: 'workflow_api',
                to: serviceId,
                color: { color: '#666666' }
            });
        }
    });
    
    // Update network
    dashboardState.network.setData({ nodes, edges });
}

// Update footer stats
function updateFooterStats() {
    document.getElementById('total-tests').textContent = dashboardState.testResults.length;
    document.getElementById('total-services').textContent = Object.keys(dashboardState.services).length;
    
    // Calculate system uptime (simulated)
    const uptime = calculateUptime();
    document.getElementById('system-uptime').textContent = uptime;
}

// Calculate uptime (simulated)
function calculateUptime() {
    const healthyServices = Object.values(dashboardState.services).filter(s => s.status?.status === 'healthy').length;
    const totalServices = Object.keys(dashboardState.services).length;
    const uptimePercentage = totalServices > 0 ? (healthyServices / totalServices) * 100 : 0;
    return `${uptimePercentage.toFixed(1)}%`;
}

// Update overall status
function updateOverallStatus() {
    const statuses = Object.values(dashboardState.services).map(s => s.status?.status || 'unknown');
    let overallStatus = 'healthy';
    
    if (statuses.includes('error')) {
        overallStatus = 'error';
    } else if (statuses.includes('warning')) {
        overallStatus = 'warning';
    } else if (statuses.includes('unknown')) {
        overallStatus = 'unknown';
    }
    
    const indicator = document.getElementById('overall-status');
    indicator.className = `status-indicator ${overallStatus}`;
    indicator.querySelector('.status-text').textContent = overallStatus.charAt(0).toUpperCase() + overallStatus.slice(1);
}

// Run all tests
async function runAllTests() {
    showAlert('info', 'Running all tests...');
    
    // In a real implementation, this would trigger the test runner
    // For now, we'll simulate running tests
    const button = document.getElementById('run-all-tests');
    button.disabled = true;
    button.textContent = 'Running...';
    
    try {
        await refreshDashboard();
        showAlert('success', 'All tests completed');
    } catch (error) {
        showAlert('error', 'Failed to run tests: ' + error.message);
    } finally {
        button.disabled = false;
        button.textContent = 'Run All Tests';
    }
}

// Show test detail modal
function showTestDetail(result) {
    const modal = document.getElementById('test-detail-modal');
    const content = document.getElementById('test-detail-content');
    
    content.innerHTML = `
        <div class="detail-section">
            <h4>Test Information</h4>
            <dl>
                <dt>Test Name</dt>
                <dd>${result.test}</dd>
                <dt>Service</dt>
                <dd>${dashboardState.services[result.service]?.name || result.service}</dd>
                <dt>Category</dt>
                <dd>${result.category}</dd>
                <dt>Status</dt>
                <dd><span class="status-badge ${result.status}">${result.status}</span></dd>
                <dt>Duration</dt>
                <dd>${result.duration}ms</dd>
                <dt>Timestamp</dt>
                <dd>${result.timestamp.toLocaleString()}</dd>
            </dl>
        </div>
        ${result.details ? `
            <div class="detail-section">
                <h4>Test Details</h4>
                <pre>${JSON.stringify(result.details, null, 2)}</pre>
            </div>
        ` : ''}
    `;
    
    modal.style.display = 'flex';
}

// Show service detail modal
function showServiceDetail(serviceId) {
    const modal = document.getElementById('service-detail-modal');
    const content = document.getElementById('service-detail-content');
    const service = dashboardState.services[serviceId];
    
    content.innerHTML = `
        <div class="detail-section">
            <h4>Service Information</h4>
            <dl>
                <dt>Name</dt>
                <dd>${service.name}</dd>
                <dt>URL</dt>
                <dd>${service.url}</dd>
                <dt>Category</dt>
                <dd>${service.category}</dd>
                <dt>Status</dt>
                <dd><span class="status-badge ${service.status?.status}">${service.status?.status || 'unknown'}</span></dd>
                <dt>Response Time</dt>
                <dd>${service.status?.responseTime ? service.status.responseTime + 'ms' : 'N/A'}</dd>
            </dl>
        </div>
        <div class="detail-section">
            <h4>Endpoints</h4>
            <dl>
                <dt>Health</dt>
                <dd>${service.health_endpoint}</dd>
                ${service.graphql_endpoint ? `
                    <dt>GraphQL</dt>
                    <dd>${service.graphql_endpoint}</dd>
                ` : ''}
                ${service.websocket_endpoint ? `
                    <dt>WebSocket</dt>
                    <dd>${service.websocket_endpoint}</dd>
                ` : ''}
            </dl>
        </div>
        <div class="detail-section">
            <h4>Recent Tests</h4>
            <table class="detail-table">
                <thead>
                    <tr>
                        <th>Time</th>
                        <th>Test</th>
                        <th>Status</th>
                        <th>Duration</th>
                    </tr>
                </thead>
                <tbody>
                    ${dashboardState.testResults
                        .filter(r => r.service === serviceId)
                        .slice(0, 5)
                        .map(r => `
                            <tr>
                                <td>${r.timestamp.toLocaleTimeString()}</td>
                                <td>${r.test}</td>
                                <td><span class="status-badge ${r.status}">${r.status}</span></td>
                                <td>${r.duration}ms</td>
                            </tr>
                        `).join('')}
                </tbody>
            </table>
        </div>
    `;
    
    modal.style.display = 'flex';
}

// Close test detail modal
function closeTestDetail() {
    document.getElementById('test-detail-modal').style.display = 'none';
}

// Close service detail modal
function closeServiceDetail() {
    document.getElementById('service-detail-modal').style.display = 'none';
}

// Show alert
function showAlert(type, message) {
    const alert = {
        id: Date.now(),
        type: type,
        message: message,
        timestamp: new Date()
    };
    
    dashboardState.alerts.unshift(alert);
    renderAlerts();
    
    // Auto-dismiss after 5 seconds
    setTimeout(() => {
        dashboardState.alerts = dashboardState.alerts.filter(a => a.id !== alert.id);
        renderAlerts();
    }, 5000);
}

// Render alerts
function renderAlerts() {
    const container = document.getElementById('alert-container');
    container.innerHTML = '';
    
    dashboardState.alerts.slice(0, 5).forEach(alert => {
        const alertEl = document.createElement('div');
        alertEl.className = `alert alert-${alert.type}`;
        alertEl.innerHTML = `
            <div class="alert-content">
                <span class="alert-message">${alert.message}</span>
                <span class="alert-time">${alert.timestamp.toLocaleTimeString()}</span>
            </div>
            <button class="alert-close" onclick="dismissAlert(${alert.id})">&times;</button>
        `;
        container.appendChild(alertEl);
    });
}

// Dismiss alert
function dismissAlert(alertId) {
    dashboardState.alerts = dashboardState.alerts.filter(a => a.id !== alertId);
    renderAlerts();
}

// Helper functions
function getServiceColor(index, opacity = 1) {
    const colors = [
        `rgba(75, 192, 192, ${opacity})`,
        `rgba(54, 162, 235, ${opacity})`,
        `rgba(255, 206, 86, ${opacity})`,
        `rgba(153, 102, 255, ${opacity})`,
        `rgba(255, 159, 64, ${opacity})`
    ];
    return colors[index % colors.length];
}

function getStatusColor(status) {
    const colors = {
        healthy: '#4CAF50',
        warning: '#FFC107',
        error: '#F44336',
        unknown: '#9E9E9E'
    };
    return colors[status] || colors.unknown;
}

// Simple YAML parser (for demo purposes)
function parseSimpleYAML(yamlText) {
    // This is a very basic parser - in production, use a proper YAML library
    const config = {
        services: {},
        test_categories: {},
        alerts: {},
        dashboard: {}
    };
    
    // For now, return a default configuration
    return getDefaultConfig();
}

// Get default configuration
function getDefaultConfig() {
    return {
        services: {
            workflow_api: {
                name: "Workflow API",
                url: "http://localhost:8080",
                health_endpoint: "/health",
                graphql_endpoint: "/api/v1/graphql",
                category: "core"
            },
            graphql_gateway: {
                name: "GraphQL Gateway",
                url: "http://localhost:4000",
                health_endpoint: "/health",
                graphql_endpoint: "/graphql",
                category: "core"
            },
            content_processing: {
                name: "Content Processing",
                url: "http://localhost:8081",
                health_endpoint: "/health",
                graphql_endpoint: "/graphql",
                category: "microservice"
            },
            knowledge_graph: {
                name: "Knowledge Graph",
                url: "http://localhost:8082",
                health_endpoint: "/health",
                graphql_endpoint: "/graphql",
                category: "microservice"
            },
            realtime_communication: {
                name: "Realtime Communication",
                url: "http://localhost:8083",
                health_endpoint: "/health",
                websocket_endpoint: "/ws",
                graphql_endpoint: "/graphql",
                category: "microservice"
            }
        }
    };
}

// Global window functions for onclick handlers
window.closeTestDetail = closeTestDetail;
window.closeServiceDetail = closeServiceDetail;
window.showTestDetail = showTestDetail;
window.dismissAlert = dismissAlert;