import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

type SystemInfo = {
  cpu: {
    total_usage: number;
    temperature: number | null;
  };
  memory: {
    total: number;
    used: number;
    usage_percent: number;
  };
  network: {
    total_upload_speed: number;
    total_download_speed: number;
  };
  timestamp: number;
};

type MonitorVisibility = {
  cpu: boolean;
  mem: boolean;
  net: boolean;
};

const formatPercent = (value: number) => `${value.toFixed(2)}%`;

const formatGB = (value: number, digits: number) =>
  `${(value / 1024 / 1024 / 1024).toFixed(digits)}`;

const formatNetSpeed = (bytesPerSec: number) => {
  const bitsPerSec = bytesPerSec * 8;
  if (bitsPerSec < 1_000) {
    return `${bitsPerSec.toFixed(0)}b`;
  }
  if (bitsPerSec < 1_000_000) {
    return `${(bitsPerSec / 1_000).toFixed(1)}Kb`;
  }
  if (bitsPerSec < 1_000_000_000) {
    return `${(bitsPerSec / 1_000_000).toFixed(1)}Mb`;
  }
  return `${(bitsPerSec / 1_000_000_000).toFixed(1)}Gb`;
};

function App() {
  const [layout, setLayout] = useState<"vertical" | "horizontal">("vertical");
  const [textColor, setTextColor] = useState("#ffffff");
  const [visibility, setVisibility] = useState<MonitorVisibility>({
    cpu: true,
    mem: true,
    net: true,
  });
  const [stats, setStats] = useState({
    cpuUsage: 0,
    cpuTemp: null as number | null,
    memUsage: 0,
    memUsed: 0,
    memTotal: 0,
    netUp: 0,
    netDown: 0,
  });
  useEffect(() => {
    let mounted = true;
    const fetchInfo = async () => {
      try {
        const info = await invoke<SystemInfo>("get_system_info");
        if (!mounted) {
          return;
        }
        setStats({
          cpuUsage: info.cpu.total_usage ?? 0,
          cpuTemp: info.cpu.temperature ?? null,
          memUsage: info.memory.usage_percent ?? 0,
          memUsed: info.memory.used ?? 0,
          memTotal: info.memory.total ?? 0,
          netUp: info.network.total_upload_speed ?? 0,
          netDown: info.network.total_download_speed ?? 0,
        });
      } catch (error) {
        console.error("Failed to fetch system info", error);
      }
    };

    fetchInfo();
    const timer = window.setInterval(fetchInfo, 1000);
    return () => {
      mounted = false;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>("layout-changed", (event) => {
      const next = event.payload;
      if (next === "horizontal" || next === "vertical") {
        setLayout(next);
      }
    })
      .then((handler) => {
        unlisten = handler;
      })
      .catch((error) => {
        console.error("Failed to listen for layout changes", error);
      });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    invoke<string>("get_layout")
      .then((value) => {
        if (value === "horizontal" || value === "vertical") {
          setLayout(value);
        }
      })
      .catch((error) => {
        console.error("Failed to load layout", error);
      });
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>("text-color-changed", (event) => {
      setTextColor(event.payload);
    })
      .then((handler) => {
        unlisten = handler;
      })
      .catch((error) => {
        console.error("Failed to listen for text color", error);
      });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    invoke<string>("get_text_color")
      .then((value) => {
        if (value) {
          setTextColor(value);
        }
      })
      .catch((error) => {
        console.error("Failed to load text color", error);
      });
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<MonitorVisibility>("monitor-visibility-changed", (event) => {
      setVisibility(event.payload);
    })
      .then((handler) => {
        unlisten = handler;
      })
      .catch((error) => {
        console.error("Failed to listen for monitor visibility", error);
      });
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    invoke<MonitorVisibility>("get_monitor_visibility")
      .then((value) => {
        setVisibility(value);
      })
      .catch((error) => {
        console.error("Failed to load monitor visibility", error);
      });
  }, []);

  const handleMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) {
      if (event.button === 2) {
        invoke("toggle_layout").catch((error) => {
          console.error("Failed to toggle layout", error);
        });
      }
      return;
    }
    getCurrentWindow().startDragging().catch((error) => {
      console.error("Failed to start dragging", error);
    });
  };

  const handleMouseUp = (event: React.MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) {
      return;
    }
    invoke("snap_window").catch((error) => {
      console.error("Failed to snap window", error);
    });
  };

  return (
    <div
      className={
        layout === "horizontal" ? "layout-horizontal" : "layout-vertical"
      }
      style={{ color: textColor }}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onContextMenu={(event) => event.preventDefault()}
    >
      {visibility.cpu && (
        <div>
          <b>CPU</b>
          <div>{formatPercent(stats.cpuUsage)}</div>
          <div>
            {stats.cpuTemp == null ? "--" : `${stats.cpuTemp.toFixed(1)}°C`}
          </div>
        </div>
      )}
      {visibility.mem && (
        <div>
          <b>Mem</b>
          <div>{formatPercent(stats.memUsage)}</div>
          <div>
            {formatGB(stats.memUsed, 1)}/{formatGB(stats.memTotal, 0)}
          </div>
        </div>
      )}
      {visibility.net && (
        <div>
          <b>Net</b>
          <div>↑{formatNetSpeed(stats.netUp)}/s</div>
          <div>↓{formatNetSpeed(stats.netDown)}/s</div>
        </div>
      )}
    </div>
  );
}

export default App;
