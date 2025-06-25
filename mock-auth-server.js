const express = require('express');
const cors = require('cors');
const app = express();

app.use(cors());
app.use(express.json());

// Mock auth endpoint
app.post('/auth/token', (req, res) => {
  const { sub, role } = req.body;
  
  // Create a mock JWT token
  const mockToken = Buffer.from(JSON.stringify({
    sub,
    role,
    exp: Date.now() + 86400000, // 24 hours
    iat: Date.now()
  })).toString('base64');
  
  res.json({
    access_token: `mock.${mockToken}.signature`,
    token_type: 'Bearer',
    expires_in: 86400
  });
});

// Mock health endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

// Mock workflow endpoints
app.get('/api/v1/workflows/templates', (req, res) => {
  res.json([
    {
      id: 'customer-support',
      name: 'Customer Support Workflow',
      description: 'Handle customer inquiries'
    },
    {
      id: 'knowledge-base',
      name: 'Knowledge Base Workflow',
      description: 'Search and retrieve information'
    }
  ]);
});

const PORT = 8080;
app.listen(PORT, () => {
  console.log(`Mock auth server running on http://localhost:${PORT}`);
});