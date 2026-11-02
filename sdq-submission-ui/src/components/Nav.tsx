import { NavLink } from "react-router";
import "./Nav.scss";

function Nav() {
  return (
    <nav className="nav">
      <NavLink to="/" end>
        Home
      </NavLink>
      <NavLink to="/page1">Page 1</NavLink>
      <NavLink to="/page2">Page 2</NavLink>
      <NavLink to="/page3">Page 3</NavLink>
    </nav>
  );
}

export default Nav;
