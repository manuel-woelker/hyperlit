import {useBookStructureStore} from "../structure/BookStructureStore.ts";

export function Layout() {
  let title = useBookStructureStore((store) => store.book.title);
  let chapters = useBookStructureStore((store) => store.book.chapters);
  return (
      <div style={{
        display: 'grid',
        gridTemplateRows: '56px 1fr',
        gridTemplateColumns: '260px 1fr',
        height: '100vh',
        width: '100vw'
      }}>
        {/* Fixed Header */}
        <header style={{
          gridColumn: '1 / span 2',
          gridRow: '1',
          position: 'sticky',
          top: 0,
          zIndex: 1000,
          display: 'flex',
          alignItems: 'center',
          padding: '0 16px',
          background: '#0f172a',
          color: 'white',
          borderBottom: '1px solid #1f2937'
        }}>
          <div style={{fontWeight: 700}}>{title} - Docs</div>
        </header>

        {/* Sidebar Navigation */}
        <aside style={{
          gridColumn: '1',
          gridRow: '2',
          overflow: 'auto',
          borderRight: '1px solid #e5e7eb',
          background: '#f8fafc',
          padding: '12px'
        }}>
          <nav>
            <ul style={{listStyle: 'none', margin: 0, padding: 0}}>
              <li>
                <details open>
                  <summary style={{cursor: 'pointer', fontWeight: 600}}>Getting Started</summary>
                  <ul style={{listStyle: 'none', margin: '8px 0 0 12px', padding: 0}}>
                    <li><a href="#intro" style={{textDecoration: 'none', color: '#111827'}}>Introduction</a></li>
                    <li><a href="#install" style={{textDecoration: 'none', color: '#111827'}}>Installation</a></li>
                  </ul>
                </details>
              </li>
              <li>
                <details>
                  <summary style={{cursor: 'pointer', fontWeight: 600}}>Guides</summary>
                  <ul style={{listStyle: 'none', margin: '8px 0 0 12px', padding: 0}}>
                    <li><a href="#config" style={{textDecoration: 'none', color: '#111827'}}>Configuration</a></li>
                    <li><a href="#themes" style={{textDecoration: 'none', color: '#111827'}}>Theming</a></li>
                  </ul>
                </details>
              </li>
              <li>
                <summary style={{cursor: 'pointer', fontWeight: 600}}>Chapters</summary>
                <ul style={{listStyle: 'none', margin: '8px 0 0 12px', padding: 0}}>
                  {chapters.map((chapter =>
                          <li><a href="#config" style={{textDecoration: 'none', color: '#111827'}}>{chapter.label}</a>
                          </li>
                  ))}

                </ul>
              </li>
            </ul>
          </nav>
        </aside>

        {/* Main Content */}
        <main style={{
          gridColumn: '2',
          gridRow: '2',
          overflow: 'auto',
          padding: '24px'
        }}>
          <article style={{maxWidth: 900, margin: '0 auto'}}>
            <h1 id="intro">Introduction</h1>
            <p>
              Welcome to the documentation. Replace this with your content component tree.
            </p>
            <h2 id="install">Installation</h2>
            <p>
              Add installation instructions here.
            </p>
            <h2 id="config">Configuration</h2>
            <p>
              Configuration guide content.
            </p>
            <h2 id="themes">Theming</h2>
            <p>
              Theming information and examples.
            </p>
          </article>
        </main>
      </div>
  );
}