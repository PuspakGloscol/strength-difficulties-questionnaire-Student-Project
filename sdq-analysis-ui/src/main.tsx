import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router";

import "./styles/index.scss";

import InProgressContextProvider from "./context/InProgressContext/InProgressContextProvider";
import ApplicationErrorContextProvider from "./context/AppNotificationContext/AppNotificationContextProvider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import AppRoutes from "./pages/AppRoutes/AppRoutes";

// You want these flags or React Query will refetch EVERYTHING if you BREATHE near it.
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      retry: false,
    },
  },
});

createRoot(document.getElementById("root")!).render(
  <BrowserRouter>
    <QueryClientProvider client={queryClient}>
      <ApplicationErrorContextProvider>
        <InProgressContextProvider>
          <AppRoutes />
        </InProgressContextProvider>
      </ApplicationErrorContextProvider>
    </QueryClientProvider>
  </BrowserRouter>,
);
