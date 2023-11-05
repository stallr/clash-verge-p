import { IconButton, Paper } from "@mui/material";
import { useLockFn } from "ahooks";
import { useTranslation } from "react-i18next";
import { BasePage, Notice } from "@/components/base";
import { Google } from "@mui/icons-material";
import { openWebUrl } from "@/services/cmds";
import SettingVerge from "@/components/setting/setting-verge";
import SettingClash from "@/components/setting/setting-clash";
import SettingSystem from "@/components/setting/setting-system";

const SettingPage = () => {
  const { t } = useTranslation();

  const onError = (err: any) => {
    Notice.error(err?.message || err.toString());
  };

  const toGoogleRepo = useLockFn(() => {
    return openWebUrl("https://www.google.com/");
  });

  return (
    <BasePage
      title={t("Settings")}
      header={
        <IconButton
          size="small"
          color="inherit"
          title=""
          onClick={toGoogleRepo}
        >
          <Google fontSize="inherit" />
        </IconButton>
      }
    >
      <Paper sx={{ borderRadius: 1, boxShadow: 2, mb: 3 }}>
        <SettingClash onError={onError} />
      </Paper>

      <Paper sx={{ borderRadius: 1, boxShadow: 2, mb: 3 }}>
        <SettingSystem onError={onError} />
      </Paper>

      <Paper sx={{ borderRadius: 1, boxShadow: 2 }}>
        <SettingVerge onError={onError} />
      </Paper>
    </BasePage>
  );
};

export default SettingPage;
