import React, { useState, useRef, useEffect, useCallback } from 'react';
import { ChatMessage } from '../../types/chat';
import { WorkflowIntent } from '../../types/workflow';
import { IntentType, WorkflowType } from '../../../../services/ai/WorkflowIntentAnalyzer';
import styles from './WorkflowPreview.module.css';

// Helper function to get node type CSS class safely
const getNodeTypeClass = (nodeType: string): string => {
  const typeMap: Record<string, string> = {
    'trigger': styles.nodeTypeTrigger,
    'source': styles.nodeTypeSource,
    'condition': styles.nodeTypeCondition,
    'action': styles.nodeTypeAction,
    'transformation': styles.nodeTypeTransformation,
    'ai': styles.nodeTypeAI,
  };
  return typeMap[nodeType] || '';
};

export interface WorkflowNode {
  id: string;
  type: 'trigger' | 'source' | 'condition' | 'action' | 'transformation' | 'ai';
  label: string;
  data: Record<string, any>;
}

export interface WorkflowConnection {
  from: string;
  to: string;
  label?: string;
}

export interface WorkflowPreviewProps {
  messages: ChatMessage[];
  intent: WorkflowIntent;
  currentStep?: string;
  onNodeClick?: (node: WorkflowNode) => void;
  onConnectionClick?: (connection: WorkflowConnection) => void;
  onNodeHover?: (nodeId: string, isHovering: boolean) => void;
  showConnectionLabels?: boolean;
  highlightMode?: 'default' | 'focus';
  enableZoomPan?: boolean;
}

export const WorkflowPreview: React.FC<WorkflowPreviewProps> = ({
  messages,
  intent,
  currentStep,
  onNodeClick,
  onConnectionClick,
  onNodeHover,
  showConnectionLabels = false,
  highlightMode = 'default',
  enableZoomPan = false,
}) => {
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; nodeId: string } | null>(null);
  const [viewMode, setViewMode] = useState<'expanded' | 'compact'>('expanded');
  const [focusedNodeId, setFocusedNodeId] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0 });
  const canvasRef = useRef<HTMLDivElement>(null);
  const nodesRef = useRef<Map<string, HTMLDivElement>>(new Map());
  const tooltipTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Build nodes from intent
  const buildNodes = useCallback((): WorkflowNode[] => {
    const nodes: WorkflowNode[] = [];
    const { extractedEntities, parameters } = intent;

    // Check if we should show empty state
    if (intent.type === IntentType.UNKNOWN && intent.confidence === 0) {
      return [];
    }

    // Add trigger node
    if (parameters?.schedule || parameters?.triggerType || intent.type === IntentType.CREATE_WORKFLOW) {
      const triggerLabel = parameters?.schedule 
        ? `Schedule: ${parameters.schedule.charAt(0).toUpperCase() + parameters.schedule.slice(1)}`
        : parameters?.triggerType === 'schedule' && parameters?.schedule
        ? `Schedule: ${parameters.schedule.charAt(0).toUpperCase() + parameters.schedule.slice(1)}`
        : 'Webhook Trigger';
        
      nodes.push({
        id: 'trigger',
        type: 'trigger',
        label: triggerLabel,
        data: { schedule: parameters?.schedule, triggerType: parameters?.triggerType }
      });
    }

    // Add source nodes (services)
    const addedSourceNodes = new Set<string>();
    if (extractedEntities?.services) {
      extractedEntities.services.forEach((service: string) => {
        if (service === 'helpscout' && !addedSourceNodes.has('helpscout')) {
          nodes.push({
            id: 'helpscout',
            type: 'source',
            label: 'HelpScout',
            data: { service: 'helpscout' }
          });
          addedSourceNodes.add('helpscout');
        } else if (service === 'notion' && !addedSourceNodes.has('notion')) {
          nodes.push({
            id: 'notion',
            type: 'source',
            label: 'Notion',
            data: { service: 'notion' }
          });
          addedSourceNodes.add('notion');
        }
      });
    }
    
    // Also check parameters.source only if no services are explicitly defined
    if (!extractedEntities?.services && parameters?.source === 'helpscout' && !addedSourceNodes.has('helpscout')) {
      nodes.push({
        id: 'helpscout',
        type: 'source',
        label: 'HelpScout',
        data: { service: 'helpscout' }
      });
    }

    // Add condition node
    if (parameters?.condition || (parameters?.conditions && parameters.conditions.length > 0)) {
      const conditionLabel = parameters?.condition === 'priority=high' 
        ? 'Priority = High' 
        : 'Condition';
      nodes.push({
        id: 'condition',
        type: 'condition',
        label: conditionLabel,
        data: { condition: parameters?.condition, conditions: parameters?.conditions }
      });
    }

    // Add transformation node
    if (extractedEntities?.hasTransformation) {
      nodes.push({
        id: 'transformation',
        type: 'transformation',
        label: 'Data Transformation',
        data: {}
      });
    }

    // Add AI node
    if (extractedEntities?.hasAI) {
      nodes.push({
        id: 'ai',
        type: 'ai',
        label: 'AI Processing',
        data: { provider: extractedEntities.aiProvider }
      });
    }

    // Add action nodes (destinations)
    const addedActionNodes = new Set<string>();
    if (extractedEntities?.services) {
      extractedEntities.services.forEach((service: string) => {
        if (service === 'slack' && !addedActionNodes.has('slack')) {
          nodes.push({
            id: 'slack',
            type: 'action',
            label: 'Send to Slack',
            data: { service: 'slack' }
          });
          addedActionNodes.add('slack');
        } else if (service === 'email' && !addedActionNodes.has('email')) {
          nodes.push({
            id: 'email',
            type: 'action',
            label: 'Send Email',
            data: { service: 'email' }
          });
          addedActionNodes.add('email');
        }
      });
    }
    
    // Also check parameters.destination only if no services are explicitly defined
    if (!extractedEntities?.services && parameters?.destination === 'slack' && !addedActionNodes.has('slack')) {
      nodes.push({
        id: 'slack',
        type: 'action',
        label: 'Send to Slack',
        data: { service: 'slack' }
      });
    }

    // Handle modifications (for MODIFY_WORKFLOW intent)
    if (intent.type === IntentType.MODIFY_WORKFLOW && extractedEntities?.modifications) {
      const mods = extractedEntities.modifications;
      mods.forEach((mod: any) => {
        if (mod.action === 'add' && mod.service === 'email') {
          const existingEmail = nodes.find(n => n.id === 'email');
          if (!existingEmail) {
            nodes.push({
              id: 'email',
              type: 'action',
              label: 'Send Email',
              data: { service: 'email', isNew: true }
            });
          }
        }
      });
    }

    return nodes;
  }, [intent]);

  // Build connections between nodes
  const buildConnections = useCallback((nodes: WorkflowNode[]): WorkflowConnection[] => {
    const connections: WorkflowConnection[] = [];
    
    if (nodes.length < 2) return connections;

    // Create a logical flow order
    const nodeOrder = ['trigger', 'helpscout', 'notion', 'condition', 'transformation', 'ai', 'slack', 'email'];
    const sortedNodes = nodes.sort((a, b) => {
      const aIndex = nodeOrder.indexOf(a.id);
      const bIndex = nodeOrder.indexOf(b.id);
      return aIndex - bIndex;
    });

    // Connect nodes in sequence
    for (let i = 0; i < sortedNodes.length - 1; i++) {
      const fromNode = sortedNodes[i];
      const toNode = sortedNodes[i + 1];
      
      let label = '';
      if (fromNode.id === 'trigger' && toNode.id === 'helpscout') {
        label = 'Ticket Data';
      }
      
      connections.push({
        from: fromNode.id,
        to: toNode.id,
        label
      });
    }

    // Handle branching for multiple conditions
    if (intent.parameters?.conditions && intent.parameters.conditions.length > 1) {
      const conditionIndex = sortedNodes.findIndex(n => n.id === 'condition');
      if (conditionIndex >= 0) {
        // For multiple conditions, we create multiple connection paths
        // even if they lead to the same destination (to show branching logic)
        const nodesAfterCondition = sortedNodes.slice(conditionIndex + 1);
        
        if (nodesAfterCondition.length > 0) {
          // Remove existing single connection from condition
          const existingConnectionIndex = connections.findIndex(c => c.from === 'condition');
          if (existingConnectionIndex >= 0) {
            connections.splice(existingConnectionIndex, 1);
          }
          
          // Create a connection for each condition to show branching
          intent.parameters.conditions.forEach((condition: any, condIndex: number) => {
            nodesAfterCondition.forEach((node, nodeIndex) => {
              connections.push({
                from: 'condition',
                to: node.id,
                label: `${condition.field} = ${condition.value}`
              });
            });
          });
        }
      }
    }

    return connections;
  }, [intent.parameters]);

  const nodes = buildNodes();
  const connections = buildConnections(nodes);

  // Calculate node positions
  const getNodePosition = (nodeId: string, index: number): { x: number; y: number } => {
    const spacing = viewMode === 'expanded' ? 200 : 150;
    const startX = 50;
    const centerY = 200;
    
    return {
      x: startX + (index * spacing),
      y: centerY
    };
  };

  // Get step order for determining completed nodes
  const getStepOrder = (): string[] => {
    return ['trigger', 'source', 'helpscout', 'notion', 'condition', 'transformation', 'ai', 'action', 'slack', 'email'];
  };

  const isNodeCompleted = (nodeId: string): boolean => {
    if (!currentStep) return false;
    const order = getStepOrder();
    const currentIndex = order.indexOf(currentStep);
    const nodeIndex = order.indexOf(nodeId);
    
    // Also check if node type matches current step
    const node = nodes.find(n => n.id === nodeId);
    if (node && currentStep === node.type) {
      return false; // Current node is not completed yet
    }
    
    return nodeIndex < currentIndex;
  };

  const isNodeActive = (nodeId: string): boolean => {
    if (!currentStep) return false;
    
    // Direct ID match
    if (currentStep === nodeId) return true;
    
    // Type match
    const node = nodes.find(n => n.id === nodeId);
    if (node && currentStep === node.type) return true;
    
    // Special case for source step matching service nodes
    if (currentStep === 'source' && (nodeId === 'helpscout' || nodeId === 'notion')) {
      return true;
    }
    
    return false;
  };

  const isConnectionActive = (connection: WorkflowConnection): boolean => {
    if (!currentStep) return false;
    
    // Check if the connection leads to the current step
    const toNode = nodes.find(n => n.id === connection.to);
    if (toNode && (currentStep === toNode.id || currentStep === toNode.type)) {
      return true;
    }
    
    // Special case for source step
    if (currentStep === 'source' && connection.to === 'helpscout') {
      return true;
    }
    
    return false;
  };

  const isConnectionCompleted = (connection: WorkflowConnection): boolean => {
    return isNodeCompleted(connection.from) && isNodeCompleted(connection.to);
  };

  // Handle node click
  const handleNodeClick = (node: WorkflowNode) => {
    if (onNodeClick) {
      onNodeClick(node);
    }
  };

  // Handle node hover
  const handleNodeHover = (nodeId: string, isHovering: boolean) => {
    if (tooltipTimeoutRef.current) {
      clearTimeout(tooltipTimeoutRef.current);
      tooltipTimeoutRef.current = null;
    }

    if (isHovering) {
      // Add a small delay before showing tooltip
      tooltipTimeoutRef.current = setTimeout(() => {
        setHoveredNode(nodeId);
      }, 100);
    } else {
      setHoveredNode(null);
    }
    
    if (onNodeHover) {
      onNodeHover(nodeId, isHovering);
    }
  };

  // Handle connection click
  const handleConnectionClick = (connection: WorkflowConnection) => {
    if (onConnectionClick) {
      onConnectionClick(connection);
    }
  };

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent, nodeId: string) => {
    const currentIndex = nodes.findIndex(n => n.id === nodeId);
    
    if (e.key === 'ArrowRight' && currentIndex < nodes.length - 1) {
      const nextNode = nodes[currentIndex + 1];
      const nextNodeElement = nodesRef.current.get(nextNode.id);
      nextNodeElement?.focus();
    } else if (e.key === 'ArrowLeft' && currentIndex > 0) {
      const prevNode = nodes[currentIndex - 1];
      const prevNodeElement = nodesRef.current.get(prevNode.id);
      prevNodeElement?.focus();
    } else if (e.key === 'Enter' || e.key === ' ') {
      const node = nodes.find(n => n.id === nodeId);
      if (node) {
        handleNodeClick(node);
      }
    }
  };

  // Handle context menu
  const handleContextMenu = (e: React.MouseEvent, nodeId: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, nodeId });
  };

  // Close context menu when clicking outside
  useEffect(() => {
    const handleClickOutside = () => setContextMenu(null);
    if (contextMenu) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [contextMenu]);

  // Handle zoom and pan
  const handleWheel = (e: React.WheelEvent) => {
    if (!enableZoomPan) return;
    
    if (e.ctrlKey) {
      e.preventDefault();
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      setZoom(prev => Math.max(0.5, Math.min(2, prev * delta)));
    }
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    if (!enableZoomPan || e.target !== canvasRef.current) return;
    
    setIsPanning(true);
    setPanStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isPanning) return;
    
    setPan({
      x: e.clientX - panStart.x,
      y: e.clientY - panStart.y
    });
  };

  const handleMouseUp = () => {
    setIsPanning(false);
  };

  // Get node details for tooltip
  const getNodeDetails = (nodeId: string): string => {
    switch (nodeId) {
      case 'helpscout':
        return 'HelpScout Integration\nFetches support tickets';
      default:
        return '';
    }
  };

  // Show empty state if no nodes
  if (nodes.length === 0) {
    return (
      <div className={styles.workflowPreview}>
        <div className={styles.emptyState} data-testid="workflow-empty-state">
          <div className={styles.emptyStateIcon}>📊</div>
          Start building your workflow
        </div>
      </div>
    );
  }

  return (
    <div className={styles.workflowPreview}>
      {/* View toggle button */}
      <div className={styles.viewToggle}>
        <button
          className={styles.viewToggleButton}
          onClick={() => setViewMode(viewMode === 'expanded' ? 'compact' : 'expanded')}
          data-testid="view-toggle-button"
        >
          {viewMode === 'compact' ? 'Expand View' : 'Compact View'}
        </button>
      </div>

      {/* Workflow container */}
      <div 
        className={`${styles.workflowContainer} ${styles[`${viewMode}View`]}`}
        data-testid="workflow-container"
      >
        {/* Canvas for nodes and connections */}
        <div
          ref={canvasRef}
          className={styles.workflowCanvas}
          data-testid="workflow-canvas"
          data-zoom={zoom.toFixed(1)}
          data-pan-x={pan.x.toString()}
          data-pan-y={pan.y.toString()}
          onWheel={handleWheel}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          style={{
            transform: enableZoomPan ? `scale(${zoom}) translate(${pan.x}px, ${pan.y}px)` : undefined,
            transformOrigin: 'top left'
          }}
        >
          {/* Render connections as SVG */}
          <svg 
            style={{ 
              position: 'absolute', 
              top: 0, 
              left: 0, 
              width: '100%', 
              height: '100%',
              pointerEvents: 'none'
            }}
          >
            {connections.map((connection, connectionIndex) => {
              const fromIndex = nodes.findIndex(n => n.id === connection.from);
              const toIndex = nodes.findIndex(n => n.id === connection.to);
              const fromPos = getNodePosition(connection.from, fromIndex);
              const toPos = getNodePosition(connection.to, toIndex);
              
              const isActive = isConnectionActive(connection);
              const isCompleted = isConnectionCompleted(connection);
              
              return (
                <g key={`${connection.from}-${connection.to}-${connectionIndex}`}>
                  <path
                    d={`M ${fromPos.x + 100} ${fromPos.y + 25} L ${toPos.x} ${toPos.y + 25}`}
                    className={`${styles.connection} ${isActive ? styles.connectionActive : ''} ${isCompleted ? styles.connectionCompleted : ''}`}
                    data-testid={`workflow-connection-${connection.from}-${connection.to}`}
                    style={{ pointerEvents: 'stroke' }}
                    onClick={() => handleConnectionClick(connection)}
                  />
                </g>
              );
            })}
          </svg>

          {/* Render nodes */}
          {nodes.map((node, index) => {
            const position = getNodePosition(node.id, index);
            const isActive = isNodeActive(node.id);
            const isCompleted = isNodeCompleted(node.id);
            const isDimmed = highlightMode === 'focus' && currentStep && !isActive;
            const isNew = node.data.isNew;
            const progress = intent.nodeProgress?.[node.id];
            
            return (
              <div
                key={node.id}
                ref={el => {
                  if (el) nodesRef.current.set(node.id, el);
                }}
                className={`
                  ${styles.node} 
                  ${getNodeTypeClass(node.type)}
                  ${isActive ? `${styles.nodeActive} ${styles.nodeHighlight} ${styles.nodePulse}` : ''}
                  ${isCompleted ? styles.nodeCompleted : ''}
                  ${isDimmed ? styles.nodeDimmed : ''}
                  ${isNew ? styles.nodeNew : ''}
                `}
                style={{
                  left: `${position.x}px`,
                  top: `${position.y}px`
                }}
                data-testid={`workflow-node-${node.id}`}
                onClick={() => handleNodeClick(node)}
                onMouseEnter={() => handleNodeHover(node.id, true)}
                onMouseLeave={() => handleNodeHover(node.id, false)}
                onContextMenu={(e) => handleContextMenu(e, node.id)}
                onKeyDown={(e) => handleKeyDown(e, node.id)}
                role="button"
                aria-label={`${node.label} node`}
                tabIndex={0}
              >
                {node.label}
                {progress !== undefined && (
                  <div
                    className={styles.nodeProgress}
                    data-testid={`node-progress-${node.id}`}
                    data-progress={progress}
                    style={{ width: `${progress}%` }}
                  />
                )}
              </div>
            );
          })}

          {/* Tooltips */}
          {hoveredNode && (
            <div
              className={styles.nodeTooltip}
              data-testid={`node-tooltip-${hoveredNode}`}
              style={{
                left: `${getNodePosition(hoveredNode, nodes.findIndex(n => n.id === hoveredNode)).x + 50}px`,
                top: `${getNodePosition(hoveredNode, nodes.findIndex(n => n.id === hoveredNode)).y - 40}px`
              }}
            >
              {getNodeDetails(hoveredNode).split('\n').map((line, i) => (
                <div key={i}>{line}</div>
              ))}
            </div>
          )}

          {/* Connection labels for showConnectionLabels */}
          {showConnectionLabels && connections.map((connection, connectionIndex) => {
            const fromIndex = nodes.findIndex(n => n.id === connection.from);
            const toIndex = nodes.findIndex(n => n.id === connection.to);
            const fromPos = getNodePosition(connection.from, fromIndex);
            const toPos = getNodePosition(connection.to, toIndex);
            
            if (!connection.label) return null;
            
            return (
              <div
                key={`label-${connection.from}-${connection.to}-${connectionIndex}`}
                className={styles.connectionLabel}
                data-testid={`connection-label-${connection.from}-${connection.to}`}
                style={{
                  left: `${(fromPos.x + toPos.x) / 2 + 50}px`,
                  top: `${(fromPos.y + toPos.y) / 2 + 10}px`
                }}
              >
                {connection.label}
              </div>
            );
          })}
        </div>
      </div>

      {/* Context menu */}
      {contextMenu && (
        <div
          className={styles.contextMenu}
          data-testid="node-context-menu"
          style={{
            left: `${contextMenu.x}px`,
            top: `${contextMenu.y}px`
          }}
        >
          <div className={styles.contextMenuItem}>Edit Node</div>
          <div className={styles.contextMenuItem}>Delete Node</div>
          <div className={styles.contextMenuItem}>Duplicate Node</div>
        </div>
      )}

      {/* Screen reader announcements */}
      {currentStep && (
        <div role="status" className={styles.srOnly}>
          Now configuring {nodes.find(n => isNodeActive(n.id))?.label || currentStep} step
        </div>
      )}
    </div>
  );
};