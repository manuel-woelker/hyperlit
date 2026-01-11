import {siteStore} from "../site/SiteStore.ts";
import styled from "styled-components";

const Title = styled.h1`
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    padding-top: 4px;
`;

export function NavigationBar() {
  let title = siteStore.select.title();
  return <div><Title>
    {title}
  </Title>
  </div>
}