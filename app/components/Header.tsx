import React from "react";
import { Flex, Spacer } from "@chakra-ui/react";
import { IconButton } from "@chakra-ui/react";
import { AddIcon, SettingsIcon } from "@chakra-ui/icons";
import { Link } from "react-router-dom";

export const Header = () => {
  return (
    <Flex direction="column" minH="100vh" p="4">
      <Link to="/">
        <AddIcon />
      </Link>
      <Link to="/settings">
        <SettingsIcon />
      </Link>
    </Flex>
  );
};
