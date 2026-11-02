import React from "react";
import InProgressSpinner from "./components/InProgressSpinner";

import SdqNav from "./components/SdqNav";
import { Outlet } from "react-router";

const AppLayout: React.FC = () => {
  return (
    <div className="container-fluid py-3">
      <SdqNav />
      <InProgressSpinner />
      <Outlet />
    </div>
  );
};

export default AppLayout;
