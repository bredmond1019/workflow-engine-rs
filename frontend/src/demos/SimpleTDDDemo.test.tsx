import React from 'react';
import { render, screen } from '@testing-library/react';
import SimpleTDDDemo from './SimpleTDDDemo';

describe('SimpleTDDDemo', () => {
  it('renders the demo correctly', () => {
    render(<SimpleTDDDemo />);

    expect(screen.getByText('🎯 TDD Success Demo')).toBeInTheDocument();
    expect(screen.getByText('174+ Tests Passing')).toBeInTheDocument();
  });

  it('displays frontend achievements', () => {
    render(<SimpleTDDDemo />);

    expect(screen.getByText('💻 Frontend TDD Success')).toBeInTheDocument();
    expect(screen.getByText('✅ ChatMessage: 5/5 tests')).toBeInTheDocument();
    expect(screen.getByText('✅ WorkflowPreview: 32/32 tests')).toBeInTheDocument();
    expect(screen.getByText('✅ GraphQL Integration: 30/30 tests')).toBeInTheDocument();
  });

  it('displays backend integration info', () => {
    render(<SimpleTDDDemo />);

    expect(screen.getByText('🔧 Backend Integration')).toBeInTheDocument();
    expect(screen.getByText('✅ GraphQL Gateway (Port 4000)')).toBeInTheDocument();
    expect(screen.getByText('✅ Apollo Federation v2')).toBeInTheDocument();
  });

  it('shows integration success section', () => {
    render(<SimpleTDDDemo />);

    expect(screen.getByText('🚀 Integration Success')).toBeInTheDocument();
    expect(screen.getByText('React Frontend')).toBeInTheDocument();
    expect(screen.getByText('GraphQL Gateway')).toBeInTheDocument();
  });

  it('displays TDD methodology', () => {
    render(<SimpleTDDDemo />);

    expect(screen.getByText('📚 TDD Methodology Applied')).toBeInTheDocument();
    expect(screen.getByText('🔴 RED')).toBeInTheDocument();
    expect(screen.getByText('🟢 GREEN')).toBeInTheDocument();
    expect(screen.getByText('🔵 REFACTOR')).toBeInTheDocument();
  });

  it('shows live demo commands', () => {
    render(<SimpleTDDDemo />);

    expect(screen.getByText('Live Demo Commands:')).toBeInTheDocument();
    expect(screen.getByText(/npm start/)).toBeInTheDocument();
    expect(screen.getByText(/cargo run --bin graphql-gateway/)).toBeInTheDocument();
  });
});