/// <reference types="vite/client" />
/// <reference types="vite-plugin-svgr/client" />
import { invoke } from "@tauri-apps/api";
import { process } from "@tauri-apps/api";
import InstallationCheckModal from "./components/base/base-app";
import "./assets/styles/index.scss";

import { ResizeObserver } from "@juggle/resize-observer";
if (!window.ResizeObserver) {
  window.ResizeObserver = ResizeObserver;
}

import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { RecoilRoot } from "recoil";
import { BrowserRouter } from "react-router-dom";
import { BaseErrorBoundary } from "./components/base";
import Layout from "./pages/_layout";
import "./services/i18n";
import getSystem from "@/utils/get-system";

const mainElementId = "root";
const container = document.getElementById(mainElementId);

if (!container) {
  throw new Error(
    `No container '${mainElementId}' found to render application`
  );
}
const Main = () => {
  const [showModal, setShowModal] = useState(false);

  useEffect(() => {
    invoke("check_if_installed_in_applications")
      .then((isInstalled) => {
        if (isInstalled as boolean) {
          console.log("App is installed in /Applications");
        } else {
          console.log("App is not installed in /Applications");
          setShowModal(true);
        }
      })
      .catch((error) => {
        console.error("Error checking installation:", error);
      });
  }, []);

  const handleMoveToApplications = async () => {
    try {
      invoke("move_to_applications");
      process.exit(0);
    } catch (error) {}
  };

  return (
    <React.StrictMode>
      {getSystem() === "macos" && showModal && (
        <InstallationCheckModal
          onMoveToApplications={handleMoveToApplications}
        />
      )}
      <RecoilRoot>
        <BaseErrorBoundary>
          <BrowserRouter>
            <Layout />
          </BrowserRouter>
        </BaseErrorBoundary>
      </RecoilRoot>
    </React.StrictMode>
  );
};

if (container) {
  createRoot(container).render(<Main />);
} else {
  throw new Error("Failed to find the root element");
}

export default Main;
