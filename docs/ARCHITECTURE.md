# Hyperlit Architecture

## Overview

Hyperlit is a developer documentation extraction and search tool that extracts specially marked documentation comments (using `📖` markers) from source code and presents them in a searchable web interface. It bridges the gap between documentation and implementation by keeping design rationale, architectural decisions, and complex logic explanations directly alongside the code they describe.

## Core Vision

Hyperlit serves three key purposes:

1. **Extract** - Parse source code files and identify documentation markers
2. **Index** - Build a searchable index of documentation with cross-references to source locations
3. **Present** - Provide a web-based interface for browsing, searching, and exploring documentation

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Entry Point                          │
│               (Main executable binary)                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
         ┌─────────────┴─────────────┐
         │                           │
    ┌────▼──────┐          ┌────────▼────────┐
    │  File     │          │   Web Server    │
    │ Scanner   │          │    (Axum/Warp)  │
    └────┬──────┘          └────────┬────────┘
         │                          │
         │     ┌──────────────┬─────┴────────┐
         │     │              │              │
    ┌────▼──┐ ┌┴──────────┐  ┌┴───────────┐ ┌┴───────────┐
    │Parser │ │ Extractor │  │ Indexer    │ │Search      │
    │Core   │ │(Language  │  │(Full-text) │ │Engine      │
    │       │ │Specific)  │  │            │ │            │
    └───────┘ └────┬──────┘  └────────┬───┘ └─────┬──────┘
                   │                  │           │
                   └─────────┬────────┴───────────┘
                             │
                        ┌────▼──────┐
                        │ Document  │
                        │ Store     │
                        │ (In-mem   │
                        │ or RocksDB)
                        └───────────┘
```

## Core Components

### 1. File Scanner
**Responsibility**: Discover and traverse source code files

- Recursively scan directories based on configuration
- Filter files by extension (rs, ts, py, go, java, etc.)
- Respect `.gitignore` and exclude patterns
- Watch for file system changes (optional for live mode)
- Report progress and statistics

**Key Types/Modules**:
- `scanner::FileScanner` - Main scanning logic
- `scanner::Walker` - File traversal with filtering
- `scanner::Filter` - Path filtering rules

### 2. Parser Core
**Responsibility**: Extract documentation markers from source code

- Identify `📖` markers in code comments
- Extract comment blocks associated with markers
- Preserve line numbers and code context
- Support multiple comment styles (`//`, `/* */`, `#`, etc.)
- Handle multi-line documentation blocks

**Key Types/Modules**:
- `parser::Parser` - Main parsing interface
- `parser::CommentExtractor` - Comment identification and extraction
- `parser::LineMapper` - Source location tracking

### 3. Language-agnostic extraction
**Responsibility**: Parse language-specific code structure

- Identify functions, classes, methods, modules
- Associate documentation with code elements
- Extract signatures and basic metadata
- Extensible design for adding language support


**Key Types/Modules**:
- `extractor::Extractor` - Language-agnostic interface

**How do we support as many different programming languages as possible?

Use the rust syntect crate to parse source code in a language agnostic manner.


### 4. Indexer
**Responsibility**: Build searchable index of extracted documentation

- Create in-memory or persistent index
- Support full-text search
- Track document relationships and cross-references
- Enable filtering by language, file, or code element type
- Maintain source location mappings

**Key Types/Modules**:
- `indexer::Indexer` - Main indexing logic
- `indexer::Document` - Indexed document representation
- `indexer::Index` - Query and search interface

### 5. Search Engine
**Responsibility**: Execute efficient searches over indexed documents

- Full-text search with term weighting
- Faceted search (filter by language, location, etc.)
- Query parsing and expansion
- Result ranking and relevance scoring
- Autocomplete suggestions

**Key Types/Modules**:
- `search::SearchEngine` - Main search interface
- `search::Query` - Query parsing and representation

### 6. Web Server
**Responsibility**: Serve web interface and API endpoints

- RESTful API for searching documentation
- Static file serving for web UI
- WebSocket support for live updates (future)
- CORS and security headers
- Structured logging and metrics

**API Endpoints** (planned):
- `GET /api/docs` - List all documents
- `GET /api/docs/:id` - Get specific document
- `GET /api/search?q=...` - Search documents
- `GET /api/files` - List source files
- `POST /api/reindex` - Trigger reindexing
- `WS /api/watch` - Live updates (future)

**Key Modules**:
- `web::server::WebServer` - HTTP server setup
- `web::routes::*` - Endpoint handlers
- `web::middleware::*` - Auth, logging, etc.

## Data Flow

### Initialization Flow
```
1. Load configuration (dirs to scan, languages, filters)
2. Initialize file scanner
3. Scan all source files
4. For each file:
   a. Parse comments and extract markers
   b. Identify code elements
   c. Extract documentation blocks
5. Build search index
6. Start web server
```

### Query Flow
```
1. User enters search query in web UI
2. Query sent to /api/search endpoint
3. Search engine parses query
4. Execute full-text search against index
5. Rank results by relevance
6. Return results with source context
7. Display in web UI with source links
```

## Data Models

### Document
```rust
pub struct Document {
    pub id: String,
    pub title: String,
    pub markdown: String,
    pub file_path: String,
    pub line_number: usize,
    pub tags: Vec<String>,
}
```

### SearchHit
```rust
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub context_snippet: String,
}
```

## Technology Choices

### Backend
- **Language**: Rust (2024 edition)
- **Web Server**: tiny_http (small, synchronous)
- **Source parsing**: syntect
- **Markdown parsing**: pulldown-cmark
- **Search**: TantiVy or similar (full-text search library)
- **Storage**:
  - In-memory for small projects
  - RocksDB for persistent storage in large projects
- **No Async Runtime, to keep complexity low**

### Frontend
- **Package Manager**: pnpm
- **Framework**: React
- **Build**: Vite or similar (for bundling)

### Development
- **Testing**: Criterion (benchmarks), expect-test (snapshots)
- **Code Quality**: clippy, rustfmt
- **Git Hooks**: Pre-commit (format, lint, test)

## Configuration

Hyperlit looks for configuration in this order:
1. CLI flags/arguments
2. `hyperlit.toml` in project root
3. Environment variables
4. Defaults

**Key Configuration Options**:
```toml
[[directory]]
paths = ["laws"]
globs = ["*.md"]

[[directory]]
paths = ["non_existent_directory"]
globs = ["*.md"]

[[directory]]
paths = ["src"]
globs = ["*.rs", "*.cpp", "*.go", "*.java", "*.py", "*.ts", "*.cs", "*.js"]

[indexing]
full_text_search = true
store_source_context = true
max_context_lines = 5

[web]
port = 3000
host = "127.0.0.1"
enable_live_reload = false
```

## Module Organization

```
crates/
├── config/
│   └── mod.rs              # Shared structures for e.g. error handling, strings, logging and tracing
├── config/
│   └── mod.rs              # Configuration loading
├── scanner/
│   ├── mod.rs              # File scanning logic
│   └── walker.rs           # Directory traversal
├── parser/
│   ├── mod.rs              # Core parsing interface
│   ├── comment.rs          # Comment extraction
│   └── location.rs         # Source mapping
├── extractor/
│   ├── mod.rs              # Language-agnostic interface
│   ├── rust.rs             # Rust-specific extraction
│   ├── typescript.rs       # TypeScript-specific extraction
│   └── language.rs         # Language identification
├── indexer/
│   ├── mod.rs              # Indexing logic
│   ├── document.rs         # Document type
│   └── store.rs            # Index storage
├── search/
│   ├── mod.rs              # Search engine interface
│   ├── query.rs            # Query parsing
│   └── ranker.rs           # Result ranking
└── web/
    ├── mod.rs              # Web server setup
    ├── routes/
    │   ├── mod.rs
    │   ├── docs.rs         # Documentation endpoints
    │   └── search.rs       # Search endpoints
    └── middleware/
        └── mod.rs          # Security, logging, etc.
```

## Extension Points

### Adding Language Support
1. Implement `extractor::Extractor` trait
2. Add language detection logic
3. Handle language-specific comment syntax
4. Register extractor in factory

### Custom Indexing Backends
1. Implement pluggable storage interface
2. Support alternative storage systems (Elasticsearch, PostgreSQL)

### Search Customization
1. Configurable ranking algorithms
2. Custom field weighting
3. Query expansion strategies

## Performance Considerations

### Indexing
- **Lazy loading** - Only parse requested files
- **Incremental updates** - Track and update changed files
- **Parallel scanning** - Use rayon for multi-threaded file processing
- **Memory-efficient parsing** - Stream-based comment extraction

### Search
- **In-memory cache** - Cache search results and frequently accessed documents
- **Index optimization** - Pre-built inverted indexes for fast queries
- **Pagination** - Limit result sets for large matches

### Web Server
- **Static asset caching** - Long-lived cache headers for UI assets
- **Gzip compression** - Compress API responses
- **Connection pooling** - Efficient database connections

## Security Considerations

- **Input validation** - Sanitize search queries and file paths
- **Path traversal** - Prevent access to files outside configured scan paths
- **XSS prevention** - Escape HTML in documentation content
- **CORS** - Restrict API access to configured origins
- **Rate limiting** - Optional rate limiting for API endpoints

## Future Enhancements

1. **Live Reload** - WebSocket support for real-time documentation updates
2. **Collaboration** - Share and comment on documentation
3. **Versioning** - Track documentation changes across commits
4. **IDE Integration** - VSCode, JetBrains plugins
5. **Analytics** - Track which documentation is viewed most
6. **AI Features** - Documentation summarization, gap detection
7. **Multi-workspace** - Aggregate docs from multiple projects
8. **Export** - Generate static HTML, PDF, or markdown documentation

## Testing Strategy

### Platform abstraction layer (PAL)

To make testing easier all interactions and side effects are encapsulated using a platform abstraction layer (PAL). This includes
 
- Filesystem access
- Access to time
- Serving HTTP requests

### Unit Tests
- Parser: document extraction, comment handling
- Extractors: language-specific code element detection
- Indexer: document storage and retrieval
- Search: query parsing, ranking algorithms

### Integration Tests
- End-to-end: file scanning → indexing → search
- Web API: HTTP endpoint behavior

### Benchmarks
- Parser performance on large files
- Search performance on large indices
- Memory usage profiling

## Development Roadmap

### Phase 1: Core (Current)
- [ ] Complete project skeleton
- [ ] Add core infrastructure for error handling
- [ ] Add core infrastructure for tracing
- [ ] Add platform abstraction layer
- [ ] Implement file scanner
- [ ] Build comment parser

### Phase 2: Indexing & Search
- [ ] Implement indexer
- [ ] Build full-text search
- [ ] Create search engine

### Phase 3: Web Interface
- [ ] Build REST API
- [ ] Create web UI
- [ ] Add configuration system

### Phase 4: Polish & Expand
- [ ] Additional language support
- [ ] Performance optimization
- [ ] Live reload capability
- [ ] Documentation and examples
