import {NavigationTree} from "./NavigationTree.tsx";
import {DocumentView} from "../document/DocumentView.tsx";
import styled from "styled-components";
import {NavigationBar} from "./NavigationBar.tsx";


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
    box-shadow: 0px 1px 4px rgba(0, 0, 0, 0.2);
`;

const Sidebar = styled.aside`
    grid-column: 1;
    grid-row: 2;
    overflow: hidden;
    border-right: 1px solid #e5e7eb;
    background: #f8fafc;
    padding: 0px;
    height: 100%;
`;

const MainContent = styled.main`
    grid-column: 2;
    grid-row: 2;
    overflow: auto;
    padding: 24px;
`;

export function Layout() {
  return (
      <LayoutDiv>
        {/* Fixed Header */}
        <TopHeader>
          <NavigationBar/>
        </TopHeader>

        {/* Sidebar Navigation */}
        <Sidebar>
          <NavigationTree/>
        </Sidebar>

        {/* Main Content */}
        <MainContent>
          <DocumentView/>
        </MainContent>
      </LayoutDiv>
  );
}