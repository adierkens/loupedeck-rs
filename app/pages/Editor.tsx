import React, { useEffect, useMemo, useState } from "react";
import {
  Button,
  SelectPicker,
  Modal,
  Input,
  Message,
  useToaster,
} from "rsuite";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { KeyLocation, PageConfig, PluginIdentifier } from "../types";

const usePlugins = () => {
  const [plugins, setPlugins] = useState<any | undefined>();

  React.useEffect(() => {
    if (plugins === undefined) {
      invoke<Array<PluginIdentifier>>("list_plugins").then(setPlugins);
    }
  }, [plugins]);

  return plugins;
};

const usePageNames = () => {
  const [pageNames, setPageNames] = useState<string[] | undefined>();

  React.useEffect(() => {
    if (pageNames === undefined) {
      invoke<Array<string>>("get_page_names").then(setPageNames);
    }
  }, [pageNames]);

  return pageNames;
};

const usePageConfig = (
  pageName: string
): [PageConfig | undefined, (next: PageConfig) => Promise<void>] => {
  const [pageConfig, setPageConfig] = useState<PageConfig | undefined>();

  React.useEffect(() => {
    if (pageConfig === undefined) {
      invoke<PageConfig>("get_page_config", { pageName }).then((updated) => {
        setPageConfig(updated);
      });
    }
  }, [pageConfig]);

  return [
    pageConfig,
    async (pageConfig: PageConfig) => {
      await invoke("set_page_config", { pageConfig });
      setPageConfig(await invoke<PageConfig>("get_page_config", { pageName }));
    },
  ];
};

const AddNewPage = (props: { onAdd: (pageName: string) => void }) => {
  const pageNames = usePageNames();
  const [newPageName, setNewPageName] = useState<string>("");
  const toaster = useToaster();

  return (
    <div>
      <Input
        value={newPageName}
        onChange={(newValue) => {
          if (pageNames?.includes(newValue)) {
            toaster.push("Page name already exists");
          }

          setNewPageName(newValue);
        }}
      />
      <Button
        disabled={!newPageName && !pageNames?.includes(newPageName)}
        onClick={() => {
          props.onAdd(newPageName);
        }}
      >
        Submit
      </Button>
    </div>
  );
};

export const Editor = () => {
  const plugins = usePlugins();

  const pageNames = usePageNames();
  const [selectedPage, setSelectedPage] = useState<string | undefined>();
  const [showAddNewPage, setShowAddNewPage] = useState<boolean>(false);

  return (
    <div>
      Editor
      <Modal
        open={showAddNewPage}
        onClose={() => {
          setShowAddNewPage(false);
        }}
      >
        <Modal.Body>
          <AddNewPage
            onAdd={async (pageName) => {
              const pageConfig: PageConfig = {
                name: pageName,
                screen: [],
              };
              await invoke("set_page_config", { pageConfig });
              setSelectedPage(pageName);
              setShowAddNewPage(false);
            }}
          />
        </Modal.Body>
      </Modal>
      <Button
        onClick={() => {
          setShowAddNewPage(true);
        }}
      >
        + Add New Page
      </Button>
      <SelectPicker
        data={pageNames?.map((p) => ({ label: p, value: p })) ?? []}
        value={selectedPage}
        onChange={(value) => setSelectedPage(value ?? undefined)}
      />
      {selectedPage && <PageEditor pageName={selectedPage} />}
    </div>
  );
};

const PageEditor = (props: { pageName: string }) => {
  const [pageConfig, setPageConfig] = usePageConfig(props.pageName);
  const plugins = usePlugins();

  if (!pageConfig || !plugins) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <ScreenSelection
        plugins={plugins}
        onChange={(pluginID) => {
          const key_location: KeyLocation = {
            x: 0,
            y: 0,
          };

          const screen = pageConfig?.screen ?? [];

          let screen_key_index = screen.findIndex(
            ([loc]) => loc.x === key_location.x && loc.y === key_location.y
          );

          screen_key_index =
            screen_key_index === -1 ? screen.length : screen_key_index;

          screen[screen_key_index] = [key_location, pluginID];

          setPageConfig({
            ...pageConfig,
            screen,
          });
        }}
      />
      <ScreenSelection
        plugins={plugins}
        onChange={(pluginID) => {
          const key_location: KeyLocation = {
            x: 1,
            y: 1,
          };

          const screen = pageConfig?.screen ?? [];

          let screen_key_index = screen.findIndex(
            ([loc]) => loc.x === key_location.x && loc.y === key_location.y
          );

          screen_key_index =
            screen_key_index === -1 ? screen.length : screen_key_index;

          screen[screen_key_index] = [key_location, pluginID];

          setPageConfig({
            ...pageConfig,
            screen,
          });
        }}
      />

      <CurrentPageSelection />
    </div>
  );
};

const ScreenSelection = (props: {
  plugins: Array<PluginIdentifier>;
  onChange: (screen: PluginIdentifier) => void;
}) => {
  const [selectedPlugin, setSelectedPlugin] = useState<string | undefined>();
  const { plugins } = props;

  return (
    <div>
      <SelectPicker
        data={plugins.map((p) => ({
          label: `${p.plugin_id} - ${p.plugin_ref}`,
          value: `${p.plugin_id} - ${p.plugin_ref}`,
        }))}
        value={selectedPlugin}
        onChange={(value) => {
          setSelectedPlugin(value ?? undefined);
          if (value) {
            const [pluginId, pluginRef] = value.split(" - ");
            props.onChange({ plugin_id: pluginId, plugin_ref: pluginRef });
          }
        }}
      />
    </div>
  );
};

const CurrentPageSelection = (props: {}) => {
  const pageNames = usePageNames();
  const [selectedPage, setSelectedPage] = useState<string | undefined>();

  return (
    <div>
      <SelectPicker
        data={pageNames?.map((p) => ({ label: p, value: p })) ?? []}
        value={selectedPage}
        onChange={async (value) => {
          setSelectedPage(value ?? undefined);

          if (value) {
            await invoke("set_active_page", { pageName: value });
          }
        }}
      />
    </div>
  );
};
