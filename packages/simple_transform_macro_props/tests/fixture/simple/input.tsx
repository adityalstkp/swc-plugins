import styled, { css } from "react-emotion";

const Test = styled.div`
  color: red;
`;

const Test2 = styled("div")`
  color: blue;
`;

const myStyle = css`
  color: red;
`;

const ListWrapper = styled.div(
  (props) => css`
    display: block;
    position: relative;
    height: ${props.loading === "yes" ? "500px" : "auto"};
  `,
);
