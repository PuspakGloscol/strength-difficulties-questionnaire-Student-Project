import { Outlet, type RouteObject } from "react-router";

import Page1 from "./components/Page1";
import Page2 from "./components/Page2";
import App from "./App";

const ROUTES: RouteObject[] = [
  {
    path: "/",
    element: (
      <App>
        <Outlet />
      </App>
    ),
    children: [
      {
        path: "/page1",
        element: <Page1 />,
      },
      {
        path: "/page2",
        element: <Page2 />,
      },
    ],
  },
];

export default ROUTES;
