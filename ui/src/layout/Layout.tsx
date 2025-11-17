import {useBookStructureStore} from "../structure/BookStructureStore.ts";
import {NavigationTree} from "./NavigationTree.tsx";
import {ChapterView} from "../chapter/ChapterView.tsx";
import styled from "styled-components";


const LayoutDiv = styled.div`
    display: grid;
    grid-template-rows: 56px 1fr;
    grid-template-columns: 260px 1fr;
    height: 100vh;
    width: 100vw;
`;

const TopHeader = styled.header`
    grid-column: 1 / span 2;
    grid-row: 1;
    position: sticky;
    top: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    padding: 0 16px;
    background: #0f172a;
    color: white;
    border-bottom: 1px solid #1f2937;
`;

const Sidebar = styled.aside`
    grid-column: 1;
    grid-row: 2;
    overflow: auto;
    border-right: 1px solid #e5e7eb;
    background: #f8fafc;
    padding: 12px;
`;

const MainContent = styled.main`
    grid-column: 2;
    grid-row: 2;
    overflow: auto;
    padding: 24px;
`;

const Title = styled.div`
    font-weight: 700;
`;

export function Layout() {
  let title = useBookStructureStore((store) => store.book.title);
  return (
      <LayoutDiv>
        {/* Fixed Header */}
        <TopHeader>
          <Title>{title}</Title>
        </TopHeader>

        {/* Sidebar Navigation */}
        <Sidebar>
          <nav>
            <NavigationTree/>
          </nav>
        </Sidebar>

        {/* Main Content */}
        <MainContent>
          <ChapterView/>
        </MainContent>
      </LayoutDiv>
  );
}