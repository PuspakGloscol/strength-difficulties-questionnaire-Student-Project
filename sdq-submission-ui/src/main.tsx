import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router";
import ROUTES from "./routes";
import "./index.scss";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const router = createBrowserRouter(ROUTES, {
  basename: "/",
});

const queryClient = new QueryClient();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
);
