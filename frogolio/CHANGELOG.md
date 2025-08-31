# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-12-19

### Added
- Initial release of Frogolio
- User authentication system with JWT tokens
- Link-in-bio page creation and management
- Lead capture system with scoring
- Click analytics and tracking
- Dashboard with comprehensive analytics
- Theme system for customization
- Avatar upload functionality
- CSRF protection middleware
- Compression middleware for performance
- Docker support with multi-stage builds
- Database migrations with SQLx
- HTMX integration for dynamic interactions
- Responsive design with Tailwind CSS

### Technical Features
- Rust web application built with Axum 0.7
- SQLite database with SQLx for type-safe queries
- Askama templates for server-side rendering
- bcrypt password hashing
- Session management with expiration
- File upload validation and processing
- Error handling with custom AppError types
- Structured logging with tracing
- Environment-based configuration

### Security
- JWT-based authentication
- CSRF protection
- Input validation and sanitization
- Secure cookie handling
- Password hashing with bcrypt
- SQL injection prevention via SQLx

## [Unreleased]

### Planned
- Unit and integration tests
- Rate limiting middleware
- Enhanced monitoring and metrics
- API documentation
- Webhook support
- Email notifications
- Advanced analytics features
