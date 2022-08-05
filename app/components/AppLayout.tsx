import React from "react";
import { Flex, Spacer } from "@chakra-ui/react";

export const AppLayout = (props: React.PropsWithChildren<unknown>) => {
  return <Flex>{props.children}</Flex>;
};
