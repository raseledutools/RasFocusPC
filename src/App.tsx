import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

// ─── Types ────────────────────────────────────────────────────────────────────
interface Preset {
  id: string;
  name: string;
  description: string;
  icon: string;
  domains: string[];
}

interface BlockingStatus {
  active: boolean;
  activePresets: string[];
  customSiteCount: number;
}

interface Schedule {
  enabled: boolean;
  start_time: string;
  end_time: string;
  days: string[];
}

type Page = "dashboard" | "presets" | "custom" | "schedule";

const DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

// ─── Dashboard Page ───────────────────────────────────────────────────────────
function DashboardPage({ status, presets }: { status: BlockingStatus; presets: Preset[] }) {
  const activePresetDetails = presets.filter((p) => status.activePresets.includes(p.id));
  const totalDomains = activePresetDetails.reduce((sum, p) => sum + p.domains.length, 0);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
      <div className={`status-banner ${status.active ? "blocking" : ""}`}>
        <div className="status-dot" />
        <div>
          <div className="status-text">
            {status.active ? "🔴 Blocking Active" : "⚪ Blocking Inactive"}
          </div>
          <div className="status-sub">
            {status.active
              ? `${activePresetDetails.length} presets, ${status.customSiteCount} custom sites`
              : "Toggle the switch in the top bar to start blocking"}
          </div>
        </div>
      </div>

      <div className="stats-row">
        <div className="stat-box">
          <div className="stat-num">{status.activePresets.length}</div>
          <div className="stat-label">Active Presets</div>
        </div>
        <div className="stat-box">
          <div className="stat-num">{status.customSiteCount}</div>
          <div className="stat-label">Custom Sites</div>
        </div>
        <div className="stat-box">
          <div className="stat-num">{totalDomains + status.customSiteCount}</div>
          <div className="stat-label">Domains Blocked</div>
        </div>
      </div>

      {activePresetDetails.length > 0 && (
        <div className="card">
          <div className="card-header">
            <div>
              <div className="card-title">Active Blocks</div>
              <div className="card-desc">Currently blocking these categories</div>
            </div>
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
            {activePresetDetails.map((p) => (
              <div
                key={p.id}
                style={{
                  padding: "6px 12px",
                  borderRadius: 20,
                  background: "var(--success-soft)",
                  border: "1px solid var(--success)",
                  fontSize: 12,
                  fontWeight: 600,
                  color: "var(--success)",
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                }}
              >
                <span>{p.icon}</span> {p.name}
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-title" style={{ marginBottom: 10 }}>How it works</div>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {[
            ["🛡️", "Modifies Windows hosts file", "Blocks at OS level — works across all browsers"],
            ["⚡", "Instant effect", "Changes apply within seconds, no reboot needed"],
            ["🔒", "Admin required", "Run RasFocus PC as Administrator for full functionality"],
          ].map(([icon, title, desc]) => (
            <div key={title} style={{ display: "flex", gap: 12, alignItems: "flex-start" }}>
              <span style={{ fontSize: 18 }}>{icon}</span>
              <div>
                <div style={{ fontSize: 13, fontWeight: 600 }}>{title}</div>
                <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{desc}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// ─── Presets Page ─────────────────────────────────────────────────────────────
function PresetsPage({
  presets,
  status,
  onTogglePreset,
}: {
  presets: Preset[];
  status: BlockingStatus;
  onTogglePreset: (id: string, enable: boolean) => void;
}) {
  return (
    <div>
      <div className="card-header" style={{ marginBottom: 16 }}>
        <div>
          <div className="card-title" style={{ fontSize: 16 }}>Block Presets</div>
          <div className="card-desc">Tap a category to enable or disable blocking</div>
        </div>
      </div>
      <div className="presets-grid">
        {presets.map((preset) => {
          const isActive = status.activePresets.includes(preset.id);
          return (
            <div
              key={preset.id}
              className={`preset-card ${isActive ? "active" : ""}`}
              onClick={() => onTogglePreset(preset.id, !isActive)}
            >
              {isActive && <div className="preset-badge">✓</div>}
              <div className="preset-icon">{preset.icon}</div>
              <div className="preset-name">{preset.name}</div>
              <div className="preset-desc">{preset.description}</div>
              <div
                style={{
                  marginTop: 10,
                  fontSize: 11,
                  color: isActive ? "var(--success)" : "var(--text-muted)",
                }}
              >
                {isActive ? `Blocking ${preset.domains.length} domains` : `${preset.domains.length} domains`}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// ─── Custom Sites Page ────────────────────────────────────────────────────────
function CustomSitesPage({
  sites,
  onAdd,
  onRemove,
}: {
  sites: string[];
  onAdd: (site: string) => void;
  onRemove: (site: string) => void;
}) {
  const [input, setInput] = useState("");

  const handleAdd = () => {
    const val = input.trim();
    if (val) {
      onAdd(val);
      setInput("");
    }
  };

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <div className="card-title" style={{ fontSize: 16, marginBottom: 4 }}>Custom Blocked Sites</div>
        <div className="card-desc">Add any website domain to block it in all browsers</div>
      </div>

      <div className="card">
        <div className="site-input-row">
          <input
            className="site-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAdd()}
            placeholder="example.com"
          />
          <button className="btn btn-primary" onClick={handleAdd}>
            + Add
          </button>
        </div>

        <div className="site-list">
          {sites.length === 0 ? (
            <div className="empty-state">No custom sites blocked yet</div>
          ) : (
            sites.map((site) => (
              <div key={site} className="site-row">
                <span className="site-domain">{site}</span>
                <button className="btn btn-danger" onClick={() => onRemove(site)}>
                  Remove
                </button>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

// ─── Schedule Page ────────────────────────────────────────────────────────────
function SchedulePage() {
  const [schedule, setSchedule] = useState<Schedule>({
    enabled: false,
    start_time: "09:00",
    end_time: "17:00",
    days: ["Mon", "Tue", "Wed", "Thu", "Fri"],
  });
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Schedule | null>("get_schedule").then((s) => {
      if (s) setSchedule(s);
    });
  }, []);

  const toggleDay = (day: string) => {
    setSchedule((prev) => ({
      ...prev,
      days: prev.days.includes(day) ? prev.days.filter((d) => d !== day) : [...prev.days, day],
    }));
  };

  const save = async () => {
    try {
      await invoke("save_schedule", { schedule });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <div className="card-title" style={{ fontSize: 16, marginBottom: 4 }}>Block Schedule</div>
        <div className="card-desc">Automatically enable blocking during specific hours</div>
      </div>

      <div className="card">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 20 }}>
          <div>
            <div style={{ fontWeight: 600 }}>Scheduled Blocking</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)" }}>Enable auto-block on schedule</div>
          </div>
          <div
            className={`toggle-indicator`}
            style={{
              background: schedule.enabled ? "var(--accent)" : undefined,
              cursor: "pointer",
            }}
            onClick={() => setSchedule((p) => ({ ...p, enabled: !p.enabled }))}
          />
        </div>

        <div className="schedule-grid">
          <div>
            <div className="field-label">Start Time</div>
            <input
              type="time"
              className="time-input"
              value={schedule.start_time}
              onChange={(e) => setSchedule((p) => ({ ...p, start_time: e.target.value }))}
            />
          </div>
          <div>
            <div className="field-label">End Time</div>
            <input
              type="time"
              className="time-input"
              value={schedule.end_time}
              onChange={(e) => setSchedule((p) => ({ ...p, end_time: e.target.value }))}
            />
          </div>
        </div>

        <div className="field-label">Active Days</div>
        <div className="days-row">
          {DAYS.map((day) => (
            <button
              key={day}
              className={`day-chip ${schedule.days.includes(day) ? "selected" : ""}`}
              onClick={() => toggleDay(day)}
            >
              {day}
            </button>
          ))}
        </div>

        <button className="btn btn-primary" onClick={save} style={{ width: "100%" }}>
          {saved ? "✓ Saved!" : "Save Schedule"}
        </button>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <div style={{ fontSize: 12, color: "var(--text-muted)", lineHeight: 1.6 }}>
          <strong style={{ color: "var(--text-secondary)" }}>Note:</strong> Scheduled blocking requires
          RasFocus PC to be running. The schedule activates blocking automatically at the start time
          and deactivates at the end time on selected days.
        </div>
      </div>
    </div>
  );
}

// ─── Root App ─────────────────────────────────────────────────────────────────
export default function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [presets, setPresets] = useState<Preset[]>([]);
  const [status, setStatus] = useState<BlockingStatus>({ active: false, activePresets: [], customSiteCount: 0 });
  const [customSites, setCustomSites] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      const s = await invoke<BlockingStatus>("get_blocking_status");
      setStatus(s);
      const sites = await invoke<string[]>("get_blocked_sites");
      setCustomSites(sites);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    invoke<Preset[]>("get_presets").then(setPresets);
    refreshStatus();
  }, [refreshStatus]);

  const toggleMaster = async () => {
    try {
      setError(null);
      await invoke("toggle_blocking", { active: !status.active });
      refreshStatus();
    } catch (e) {
      setError(String(e));
    }
  };

  const togglePreset = async (id: string, enable: boolean) => {
    try {
      setError(null);
      await invoke("apply_preset", { presetId: id, enable });
      refreshStatus();
    } catch (e) {
      setError(String(e));
    }
  };

  const addSite = async (site: string) => {
    try {
      setError(null);
      await invoke("add_site", { site });
      refreshStatus();
    } catch (e) {
      setError(String(e));
    }
  };

  const removeSite = async (site: string) => {
    try {
      setError(null);
      await invoke("remove_site", { site });
      refreshStatus();
    } catch (e) {
      setError(String(e));
    }
  };

  const navItems: { id: Page; icon: string; label: string }[] = [
    { id: "dashboard", icon: "📊", label: "Dashboard" },
    { id: "presets", icon: "🛡️", label: "Block Presets" },
    { id: "custom", icon: "✏️", label: "Custom Sites" },
    { id: "schedule", icon: "🕐", label: "Schedule" },
  ];

  return (
    <div className="app-layout">
      {/* Topbar */}
      <div className="topbar">
        <span className="topbar-logo">RasFocus PC</span>
        <span className="topbar-subtitle">Website Blocker</span>
        <div className="topbar-spacer" />
        {error && (
          <div
            style={{
              background: "var(--danger-soft)",
              border: "1px solid var(--danger)",
              borderRadius: 6,
              padding: "6px 12px",
              fontSize: 11,
              color: "var(--danger)",
              maxWidth: 340,
              lineHeight: 1.5,
            }}
          >
            ⚠️ <strong>Admin access needed.</strong> Right-click RasFocus PC → "Run as Administrator"
          </div>
        )}
        <div
          className={`master-toggle ${status.active ? "active" : ""}`}
          onClick={toggleMaster}
        >
          <div className="toggle-indicator" />
          <span className="toggle-label">{status.active ? "Blocking ON" : "Blocking OFF"}</span>
        </div>
      </div>

      {/* Sidebar */}
      <div className="sidebar">
        <div className="sidebar-section">Navigation</div>
        {navItems.map((item) => (
          <div
            key={item.id}
            className={`nav-item ${page === item.id ? "active" : ""}`}
            onClick={() => setPage(item.id)}
          >
            <span className="nav-icon">{item.icon}</span>
            {item.label}
          </div>
        ))}
      </div>

      {/* Content */}
      <div className="content">
        {page === "dashboard" && <DashboardPage status={status} presets={presets} />}
        {page === "presets" && (
          <PresetsPage presets={presets} status={status} onTogglePreset={togglePreset} />
        )}
        {page === "custom" && (
          <CustomSitesPage sites={customSites} onAdd={addSite} onRemove={removeSite} />
        )}
        {page === "schedule" && <SchedulePage />}
      </div>
    </div>
  );
}
