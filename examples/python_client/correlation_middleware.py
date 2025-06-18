"""
Correlation ID Middleware for FastAPI applications.

This middleware ensures every request has a correlation ID for distributed tracing
and log correlation across service boundaries.
"""

import uuid
import logging
from typing import Optional, Callable
from contextvars import ContextVar

from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp, Receive, Scope, Send

# Context variable to store correlation ID for the current request
correlation_id_ctx: ContextVar[Optional[str]] = ContextVar('correlation_id', default=None)

# Common header names for correlation IDs
CORRELATION_ID_HEADERS = [
    'X-Correlation-ID',
    'X-Request-ID',
    'X-Trace-ID',
    'Correlation-ID',
    'Request-ID'
]


class CorrelationIdMiddleware(BaseHTTPMiddleware):
    """
    Middleware that ensures every request has a correlation ID.
    
    If the incoming request has a correlation ID in one of the known headers,
    it will be used. Otherwise, a new UUID will be generated.
    
    The correlation ID is:
    - Stored in context for logging
    - Added to the response headers
    - Available throughout the request lifecycle
    """
    
    def __init__(
        self,
        app: ASGIApp,
        header_name: str = 'X-Correlation-ID',
        generator: Optional[Callable[[], str]] = None,
        validate: bool = True
    ):
        """
        Initialize the correlation ID middleware.
        
        Args:
            app: The ASGI application
            header_name: The header name to use for correlation ID
            generator: Optional custom ID generator function
            validate: Whether to validate incoming correlation IDs
        """
        super().__init__(app)
        self.header_name = header_name
        self.generator = generator or self._generate_correlation_id
        self.validate = validate
    
    @staticmethod
    def _generate_correlation_id() -> str:
        """Generate a new correlation ID using UUID4."""
        return str(uuid.uuid4())
    
    @staticmethod
    def _is_valid_correlation_id(correlation_id: str) -> bool:
        """
        Validate a correlation ID.
        
        Basic validation to ensure it's not empty and has reasonable length.
        """
        if not correlation_id or not isinstance(correlation_id, str):
            return False
        
        # Basic length validation (UUID is 36 chars with dashes)
        if len(correlation_id) < 1 or len(correlation_id) > 128:
            return False
        
        # Check if it contains only allowed characters
        allowed_chars = set('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.')
        return all(c in allowed_chars for c in correlation_id)
    
    def _get_correlation_id_from_request(self, request: Request) -> Optional[str]:
        """
        Extract correlation ID from request headers.
        
        Checks multiple common header names for correlation ID.
        """
        for header in CORRELATION_ID_HEADERS:
            correlation_id = request.headers.get(header)
            if correlation_id:
                if self.validate and not self._is_valid_correlation_id(correlation_id):
                    logging.warning(f"Invalid correlation ID received: {correlation_id}")
                    return None
                return correlation_id
        return None
    
    async def dispatch(self, request: Request, call_next):
        """
        Process the request and ensure it has a correlation ID.
        """
        # Get or generate correlation ID
        correlation_id = self._get_correlation_id_from_request(request)
        if not correlation_id:
            correlation_id = self.generator()
            logging.debug(f"Generated new correlation ID: {correlation_id}")
        else:
            logging.debug(f"Using existing correlation ID: {correlation_id}")
        
        # Set correlation ID in context
        correlation_id_ctx.set(correlation_id)
        
        # Add correlation ID to request state
        request.state.correlation_id = correlation_id
        
        # Process the request
        response = await call_next(request)
        
        # Add correlation ID to response headers
        response.headers[self.header_name] = correlation_id
        
        return response


class LoggingFilter(logging.Filter):
    """
    Logging filter that adds correlation ID to log records.
    """
    
    def filter(self, record: logging.LogRecord) -> bool:
        """Add correlation ID to log record."""
        record.correlation_id = get_correlation_id() or 'no-correlation-id'
        return True


def get_correlation_id() -> Optional[str]:
    """
    Get the current correlation ID from context.
    
    Returns:
        The current correlation ID or None if not set
    """
    return correlation_id_ctx.get()


def set_correlation_id(correlation_id: str) -> None:
    """
    Set the correlation ID in the current context.
    
    Args:
        correlation_id: The correlation ID to set
    """
    correlation_id_ctx.set(correlation_id)


def configure_logging_with_correlation_id(
    log_format: Optional[str] = None,
    log_level: str = 'INFO'
) -> None:
    """
    Configure logging to include correlation IDs.
    
    Args:
        log_format: Custom log format string
        log_level: Logging level
    """
    if log_format is None:
        log_format = (
            '[%(asctime)s] [%(correlation_id)s] %(levelname)s - '
            '%(name)s - %(message)s'
        )
    
    # Remove existing handlers
    root_logger = logging.getLogger()
    for handler in root_logger.handlers[:]:
        root_logger.removeHandler(handler)
    
    # Create new handler with correlation ID filter
    handler = logging.StreamHandler()
    handler.setFormatter(logging.Formatter(log_format))
    handler.addFilter(LoggingFilter())
    
    # Configure root logger
    root_logger.addHandler(handler)
    root_logger.setLevel(getattr(logging, log_level.upper()))


def inject_correlation_id_header(headers: dict, correlation_id: Optional[str] = None) -> dict:
    """
    Inject correlation ID into outgoing request headers.
    
    Args:
        headers: Dictionary of headers to modify
        correlation_id: Optional correlation ID to use (defaults to current context)
    
    Returns:
        Updated headers dictionary
    """
    if correlation_id is None:
        correlation_id = get_correlation_id()
    
    if correlation_id:
        headers['X-Correlation-ID'] = correlation_id
    
    return headers