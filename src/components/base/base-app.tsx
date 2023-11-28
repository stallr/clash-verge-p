// InstallationCheckModal.tsx
import React from "react";
import { appWindow } from "@tauri-apps/api/window";
import { process } from "@tauri-apps/api";
import { t } from "i18next";
import { Button, Stack, Typography } from "@mui/material";

interface InstallationCheckModalProps {
  onMoveToApplications: () => void;
}

const InstallationCheckModal: React.FC<InstallationCheckModalProps> = ({
  onMoveToApplications,
}) => {
  const handleExitApp = () => {
    process.exit(0);
  };
  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: "rgba(0, 0, 0, 0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
    >
      <div
        style={{
          backgroundColor: "white",
          padding: "20px",
          borderRadius: "5px",
          zIndex: 1001,
        }}
      >
        <p>
          {t!(
            "Your application is not installed in the Applications folder.",
            "请将软件移动至 Application 应用列表再打开"
          )}
        </p>
        <br></br>
        {/* <Button variant="contained" onClick={onMoveToApplications}>{t!("Move to Applications", "移动至 Application 应用列表")}</Button>
            <br></br><br></br> */}
        <Button variant="outlined" onClick={handleExitApp}>
          {t!("Quit", "退出")}
        </Button>
      </div>
    </div>
  );
};

export default InstallationCheckModal;
