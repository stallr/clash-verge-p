import useSWR from "swr";
import { useRef } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { useTranslation } from "react-i18next";
import { IconButton, Switch } from "@mui/material";
import { Lock } from "@mui/icons-material";
import { ArrowForward, PrivacyTipRounded, Settings } from "@mui/icons-material";
import {
  checkService,
  grantPermission,
  installService,
  patchVergeConfig,
  restartSidecar,
} from "@/services/cmds";
import { useVerge } from "@/hooks/use-verge";
import { useLockFn } from "ahooks";
import { DialogRef, Notice } from "@/components/base";
import { SettingList, SettingItem } from "./mods/setting-comp";
import { GuardState } from "./mods/guard-state";
import { ServiceViewer } from "./mods/service-viewer";
import { SysproxyViewer } from "./mods/sysproxy-viewer";
import getSystem from "@/utils/get-system";

interface Props {
  onError?: (err: Error) => void;
}

const isWIN = getSystem() === "windows";

const SettingSystem = ({ onError }: Props) => {
  const { t } = useTranslation();

  const { verge, mutateVerge, patchVerge } = useVerge();

  // service mode
  const { data: serviceStatus } = useSWR(
    isWIN ? "checkService" : null,
    checkService,
    {
      revalidateIfStale: false,
      shouldRetryOnError: false,
      focusThrottleInterval: 36e5, // 1 hour
    }
  );

  const OS = getSystem();
  const { clash_core = "clash" } = verge ?? {};
  const onGrant = useLockFn(async (core: string) => {
    try {
      await grantPermission(core);
      // 自动重启
      if (core === clash_core) await restartSidecar();
      Notice.success(`Successfully grant permission to ${core}`, 1000);
    } catch (err: any) {
      Notice.error(err?.message || err.toString());
    }
  });
  const onGuard = async (e: boolean) => {
    if (OS === "macos" || OS === "linux") {
      try {
        await grantPermission(clash_core);
      } catch (err: any) {
        Notice.error(err?.message || err.toString());
        return false;
      }
      await restartSidecar();
    } else {
      try {
        const result = await invoke<any>("check_service");
        if (result?.code !== 0 && result?.code !== 400) {
          await installService();
        }
        await patchVergeConfig({ enable_service_mode: true });
      } catch (err: any) {
        try {
          await installService();
          await patchVergeConfig({ enable_service_mode: true });
        } catch (err: any) {
          Notice.error(err?.message || err.toString());
          return false;
        }
      }
    }
    return true;
  };
  const serviceRef = useRef<DialogRef>(null);
  const sysproxyRef = useRef<DialogRef>(null);

  const {
    enable_tun_mode,
    enable_auto_launch,
    enable_service_mode,
    enable_silent_start,
    enable_system_proxy,
  } = verge ?? {};

  const onSwitchFormat = (_e: any, value: boolean) => value;
  const onChangeData = (patch: Partial<IVergeConfig>) => {
    mutateVerge({ ...verge, ...patch }, false);
  };

  return (
    <SettingList title={t("System Setting")}>
      <SysproxyViewer ref={sysproxyRef} />
      {isWIN && (
        <ServiceViewer ref={serviceRef} enable={!!enable_service_mode} />
      )}

      <SettingItem label={t("Tun Mode")}>
        {OS === "linux" && (
          <IconButton
            color="inherit"
            size="small"
            edge="start"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              onGrant(clash_core);
            }}
          >
            <Lock fontSize="inherit" />
          </IconButton>
        )}
        <GuardState
          value={enable_tun_mode ?? false}
          valueProps="checked"
          onCatch={onError}
          onFormat={onSwitchFormat}
          onChange={async (e) => {
            if (e === false || (await onGuard(e))) {
              patchVerge({ enable_tun_mode: e });
              onChangeData({ enable_tun_mode: e });
            }
          }}
        >
          <Switch edge="end" />
        </GuardState>
      </SettingItem>

      {isWIN && (
        <SettingItem
          label={t("Service Mode")}
          extra={
            <IconButton
              color="inherit"
              size="small"
              onClick={() => serviceRef.current?.open()}
            >
              <PrivacyTipRounded
                fontSize="inherit"
                style={{ cursor: "pointer", opacity: 0.75 }}
              />
            </IconButton>
          }
        >
          <GuardState
            value={enable_service_mode ?? false}
            valueProps="checked"
            onCatch={onError}
            onFormat={onSwitchFormat}
            onChange={(e) => onChangeData({ enable_service_mode: e })}
            onGuard={(e) => patchVerge({ enable_service_mode: e })}
          >
            <Switch
              edge="end"
              disabled={
                serviceStatus !== "active" && serviceStatus !== "installed"
              }
            />
          </GuardState>
        </SettingItem>
      )}

      <SettingItem
        label={t("System Proxy")}
        extra={
          <IconButton
            color="inherit"
            size="small"
            onClick={() => sysproxyRef.current?.open()}
          >
            <Settings
              fontSize="inherit"
              style={{ cursor: "pointer", opacity: 0.75 }}
            />
          </IconButton>
        }
      >
        <GuardState
          value={enable_system_proxy ?? false}
          valueProps="checked"
          onCatch={onError}
          onFormat={onSwitchFormat}
          onChange={(e) => onChangeData({ enable_system_proxy: e })}
          onGuard={(e) => patchVerge({ enable_system_proxy: e })}
        >
          <Switch edge="end" />
        </GuardState>
      </SettingItem>

      <SettingItem label={t("Auto Launch")}>
        <GuardState
          value={enable_auto_launch ?? false}
          valueProps="checked"
          onCatch={onError}
          onFormat={onSwitchFormat}
          onChange={(e) => onChangeData({ enable_auto_launch: e })}
          onGuard={(e) => patchVerge({ enable_auto_launch: e })}
        >
          <Switch edge="end" />
        </GuardState>
      </SettingItem>

      <SettingItem label={t("Silent Start")}>
        <GuardState
          value={enable_silent_start ?? false}
          valueProps="checked"
          onCatch={onError}
          onFormat={onSwitchFormat}
          onChange={(e) => onChangeData({ enable_silent_start: e })}
          onGuard={(e) => patchVerge({ enable_silent_start: e })}
        >
          <Switch edge="end" />
        </GuardState>
      </SettingItem>
    </SettingList>
  );
};

export default SettingSystem;
