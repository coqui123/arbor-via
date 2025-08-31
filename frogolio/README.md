# Frogolio: A Modern Link-in-Bio & Lead-Capture Platform

Frogolio is a production-ready Rust web application that lets creators publish personal link hubs—called *frogols*—while silently capturing click-stream analytics and qualified leads. Built with Axum, Askama, HTMX, and SQLx, it provides a complete solution for creators to manage their online presence and grow their audience.

## 🚀 Features

### Core Functionality
- **Link-in-Bio Pages**: Create beautiful, customizable link pages
- **Lead Capture**: Silent email collection with scoring system
- **Click Analytics**: Track link performance and visitor behavior
- **User Authentication**: Secure JWT-based authentication system
- **Dashboard**: Comprehensive analytics and management interface
- **Theme System**: Customizable themes for personalization
- **Drag & Drop**: Reorder links with intuitive interface

### Technical Stack
- **Backend**: Rust with Axum 0.7 web framework
- **Templates**: Askama 0.12 for compile-time template rendering
- **Frontend**: HTMX for dynamic interactions without JavaScript
- **Database**: SQLite with SQLx for type-safe queries
- **Authentication**: JWT tokens with bcrypt password hashing
- **Styling**: Tailwind CSS for modern, responsive design

## 🛠️ Installation

### Prerequisites
- Rust 1.70+ and Cargo
- Git

### Setup
1. Clone the repository:
```bash
git clone https://github.com/yourusername/frogolio.git
cd frogolio
```

2. Set up environment variables:
```bash
# Create .env file
echo "DATABASE_URL=sqlite://frogolio.db" > .env
echo "JWT_SECRET=your-super-secret-jwt-key-change-in-production" >> .env
```

3. Install dependencies and run migrations:
```bash
cargo build
cargo sqlx migrate run
```

4. Configure environment:
```bash
# Option A: .env (recommended)
echo "DATABASE_URL=sqlite://frogolio.db" > .env
echo "JWT_SECRET=change-me" >> .env

# Option B: shell env (PowerShell)
$env:DATABASE_URL = "sqlite://frogolio.db"
$env:JWT_SECRET = "change-me"

# Option B: shell env (Git Bash)
export DATABASE_URL=sqlite://frogolio.db
export JWT_SECRET=change-me
```

5. Start the server:
```bash
cargo run
```

The application will be available at `http://localhost:3000`

## 📖 Usage

### For Creators

1. **Register an Account**
   - Visit `/register` to create your account
   - Use a valid email and secure password

2. **Create Your First Frogol**
   - Log in to access your dashboard
   - Click "Create New Frogol"
   - Choose a unique slug and display name
   - Customize your theme

3. **Add Your Links**
   - Use the drag-and-drop interface to add links
   - Reorder links by dragging them
   - Edit or delete links as needed

4. **Share Your Frogol**
   - Your frogol is available at `http://localhost:3000/your-slug`
   - Share this URL in your social media bios
   - Visitors can click links and subscribe to your list

### For Visitors

1. **Browse Links**
   - Visit any frogol URL (e.g., `/john-doe`)
   - Click on links to visit external sites
   - All clicks are tracked for analytics

2. **Subscribe to Lists**
   - Use the email capture form on frogol pages
   - Get added to the creator's lead database
   - Receive updates and content from creators

## 🏗️ Architecture

### Project Structure
```
src/
├── main.rs              # Application entry point
├── state.rs             # Application state management
├── errors.rs            # Error handling
├── routes/              # HTTP route handlers
│   ├── auth.rs          # Authentication routes
│   ├── dashboard.rs     # Dashboard routes
│   ├── frogol.rs        # Frogol management
│   └── lead.rs          # Lead capture
├── services/            # Business logic
│   ├── auth_service.rs  # Authentication logic
│   ├── frogol_service.rs # Frogol management
│   └── lead_service.rs  # Lead processing
├── repo/                # Data access layer
│   ├── user_repo.rs     # User data operations
│   ├── frogol_repo.rs   # Frogol data operations
│   ├── link_repo.rs     # Link data operations
│   ├── lead_repo.rs     # Lead data operations
│   └── click_repo.rs    # Click tracking
└── middleware/          # HTTP middleware
    └── csrf.rs          # CSRF protection

templates/               # Askama templates
├── base.html           # Base layout
├── frogol.html         # Frogol page template
├── auth/               # Authentication templates
└── dashboard/          # Dashboard templates

migrations/             # Database migrations
static/                 # Static assets
```

### Database Schema
- **users**: Account information and authentication
- **frogols**: Link page configurations
- **links**: Individual links within frogols
- **leads**: Captured email addresses and metadata
- **clicks**: Click tracking and analytics
- **sessions**: JWT session management

## 🔧 Configuration

### Environment Variables
- `DATABASE_URL`: SQLite database connection string
- `JWT_SECRET`: Secret key for JWT token signing
- `RUST_LOG`: Logging level (default: `frogolio=debug`)

### Database Migrations
Run migrations with:
```bash
cargo sqlx migrate run
```

Create new migrations with:
```bash
cargo sqlx migrate add migration_name
```

## 🚀 Deployment

### Local Development
```bash
cargo run
```

### Production Deployment

#### Using Docker
```bash
docker build -t frogolio .
docker run -p 3000:3000 -e DATABASE_URL=sqlite:///data/frogolio.db frogolio
```

#### Using Fly.io
```bash
fly deploy
```

#### Using Railway
```bash
railway up
```

## 📊 Analytics

### Dashboard Features
- **Overview**: Total frogols, leads, and clicks
- **Frogol Management**: Create, edit, and delete frogols
- **Link Analytics**: Track individual link performance
- **Lead Management**: View and export captured leads
- **Click Statistics**: Daily, weekly, and monthly metrics

### Lead Scoring
Leads are automatically scored based on:
- **Source**: Direct traffic (100), Social media (80), Referrals (90)
- **Engagement**: Click depth and interaction patterns
- **Recency**: Recent activity boosts scores

## 🔒 Security

### Authentication
- JWT-based authentication with secure cookies
- bcrypt password hashing
- Session management with expiration

### CSRF Protection
- Built-in CSRF middleware
- Secure form handling
- SameSite cookie attributes

### Data Protection
- Input validation and sanitization
- SQL injection prevention via SQLx
- XSS protection through template escaping

## 🎨 Customization

### Themes
Themes are stored per frogol and can be customized:
- Color schemes
- Typography
- Layout variations
- Custom CSS support

### Templates
All templates use Askama and can be customized:
- Modify `templates/` directory
- Add new themes in `static/css/themes.css`
- Extend functionality with new routes

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) for the web framework
- [Askama](https://github.com/djc/askama) for template rendering
- [HTMX](https://htmx.org/) for dynamic interactions
- [SQLx](https://github.com/launchbadge/sqlx) for database operations

## 📞 Support

For support, please open an issue on GitHub or contact the maintainers.

---

**Frogolio**: Where creators connect with their audience, one link at a time. 