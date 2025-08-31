use tower_http::compression::{
    CompressionLayer, 
    predicate::{SizeAbove, DefaultPredicate, NotForContentType, Predicate}
};

/// Configure response compression with intelligent filtering
/// 
/// This middleware applies gzip, deflate, and br (brotli) compression to:
/// - Responses larger than 1KB to avoid overhead for small responses
/// - Text-based content types (HTML, CSS, JS, JSON, XML, SVG)
/// - Excludes already-compressed formats (images, videos, archives)
/// - Client requests that support compression via Accept-Encoding header
pub fn create_compression_layer() -> CompressionLayer<impl Predicate> {
    CompressionLayer::new()
        .compress_when(
            DefaultPredicate::new()
                // Only compress responses larger than 1KB to avoid overhead
                .and(SizeAbove::new(1024))
                // Don't compress already-compressed formats
                .and(NotForContentType::new("image/"))
                .and(NotForContentType::new("video/"))
                .and(NotForContentType::new("audio/"))
                .and(NotForContentType::new("application/zip"))
                .and(NotForContentType::new("application/gzip"))
                .and(NotForContentType::new("application/x-rar"))
                .and(NotForContentType::new("application/x-7z"))
                .and(NotForContentType::new("application/x-tar"))
                .and(NotForContentType::new("application/pdf"))
                .and(NotForContentType::new("font/"))
                .and(NotForContentType::new("application/font-"))
        )
}

/// Create a more aggressive compression layer for static assets
/// where we know the content types and can afford slightly higher CPU usage
pub fn create_static_compression_layer() -> CompressionLayer<SizeAbove> {
    CompressionLayer::new()
        // For static files, use lower threshold since we know they're compressible
        .compress_when(SizeAbove::new(512))
}

/// Create a lightweight compression layer for API responses
/// Optimized for JSON and small text responses
pub fn create_api_compression_layer() -> CompressionLayer<SizeAbove> {
    CompressionLayer::new()
        // Higher threshold for API responses to avoid overhead
        .compress_when(SizeAbove::new(2048))
}

 