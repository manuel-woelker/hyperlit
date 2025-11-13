import {useBookStructureStore} from "../structure/BookStructureStore.ts";
import {NavigationTree} from "./NavigationTree.tsx";
import {ChapterView} from "../chapter/ChapterView.tsx";

export function Layout() {
  let title = useBookStructureStore((store) => store.book.title);
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
          <div style={{fontWeight: 700}}>{title}</div>
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
            <NavigationTree/>
          </nav>


        </aside>

        {/* Main Content */}
        <main style={{
          gridColumn: '2',
          gridRow: '2',
          overflow: 'auto',
          padding: '24px'
        }}>
          <ChapterView/>
        </main>
      </div>
  );
}