import { Box, Grid, IconButton, Paper } from "@mui/material";
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
      <Grid container spacing={{ xs: 1, lg: 1 }}>
        <Grid item xs={6} md={6}>
          <Box sx={{ borderRadius: 1, boxShadow: 2, marginBottom: 1 }}>
            <SettingSystem onError={onError} />
          </Box>
          <Box sx={{ borderRadius: 1, boxShadow: 2 }}>
            <SettingClash onError={onError} />
          </Box>
        </Grid>
        <Grid item xs={6} md={6}>
          <Box sx={{ borderRadius: 1, boxShadow: 2 }}>
            <SettingVerge onError={onError} />
          </Box>
        </Grid>
      </Grid>
    </BasePage>
  );
};

export default SettingPage;
