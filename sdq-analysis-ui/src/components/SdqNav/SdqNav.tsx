import React from "react";
import { Link } from "react-router";
import AppNotifications from "@/components/AppNotifications";

import "./SdqNav.css";

const SdqNav: React.FC = () => {
  const onClickLogout = () => {
    window.location.href = "/logout";
  };

  return (
    <nav className="nav">
      <div className="nav-left">
        <span className="nav-title">Strength & Difficulties Analysis</span>
      </div>
      <ul className="nav-links">
        <li>
          <Link to="/">Home</Link>
        </li>
        <li>
          <Link to="/gbo">GBO</Link>
        </li>
        <li>
          <Link to="/sdq">SDQ</Link>
        </li>
        <li>
          <Link to="/demographic-report">Demographic Report</Link>
        </li>
        <li>
          <Link to="/bulk-upload">Bulk Upload</Link>
        </li>
      </ul>
      <div className="nav-right">
        <AppNotifications />
        <button onClick={onClickLogout}>Logout</button>
      </div>
    </nav>
  );
};

export default SdqNav;
