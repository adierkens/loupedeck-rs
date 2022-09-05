import React from "react";
import { Link, useLocation } from "react-router-dom";
import { Sidenav, Nav, Sidebar } from "rsuite";
import { Dashboard, Gear, Plus } from "@rsuite/icons";
import { Connect } from "./Connect";

const HeaderItems = [
  // {
  //   icon: <Dashboard />,
  //   label: "Home",
  //   to: "/",
  // },

  {
    icon: <Gear />,
    label: "Event Viewer",
    to: "/event-viewer",
  },
  {
    icon: <Gear />,
    label: "Editor",
    to: "/editor",
  },
  {
    icon: <Plus />,
    label: "Plugins",
    to: "/plugins",
  },
  {
    icon: <Gear />,
    label: "Settings",
    to: "/settings",
  },
];

export const SideNav = () => {
  const location = useLocation();
  const [expanded, setExpanded] = React.useState(true);

  return (
    <Sidebar
      style={{ display: "flex", flexDirection: "column" }}
      width={expanded ? 260 : 56}
      collapsible
    >
      <Sidenav expanded={expanded} style={{ minHeight: "100vh" }}>
        <Sidenav.Body>
          <Connect />
          <Nav>
            {HeaderItems.map((item) => (
              <Nav.Item
                as={Link}
                to={item.to}
                active={location.pathname === item.to}
                key={item.to}
                icon={item.icon}
              >
                {item.label}
              </Nav.Item>
            ))}
            <Nav.Item divider />
          </Nav>
        </Sidenav.Body>
        <Sidenav.Toggle
          expanded={expanded}
          onToggle={(expanded) => setExpanded(expanded)}
        />
      </Sidenav>
    </Sidebar>
  );
};
