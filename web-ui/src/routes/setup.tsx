import { createRoute } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { rootRoute } from "./__root";
import { configQuery } from "../api/queries";
import { api } from "../api/client";

export const setupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/setup",
  component: SetupPage,
});

function SetupPage() {
  const queryClient = useQueryClient();
  const { data: config } = useQuery(configQuery);
  const [newDir, setNewDir] = useState("");
  const [discovering, setDiscovering] = useState(false);
  const [discoveredDirs, setDiscoveredDirs] = useState<
    { path: string; repo_count: number; repo_names: string[]; already_configured: boolean }[] | null
  >(null);
  const [detectingEmails, setDetectingEmails] = useState(false);
  const [syncStatus, setSyncStatus] = useState("");

  if (!config) return null;

  const showWelcome = !config.config_exists || Object.keys(config.email_tags || {}).length === 0;

  const handleAddDir = async () => {
    const path = newDir.trim();
    if (!path) return;
    const currentDirs = (config.scan_dirs || []).map((d) => d.path);
    currentDirs.push(path);
    await api.updateConfig({ scan_dirs: currentDirs });
    setNewDir("");
    queryClient.invalidateQueries({ queryKey: ["config"] });
  };

  const handleRemoveDir = async (path: string) => {
    const currentDirs = (config.scan_dirs || []).map((d) => d.path).filter((d) => d !== path);
    await api.updateConfig({ scan_dirs: currentDirs });
    queryClient.invalidateQueries({ queryKey: ["config"] });
  };

  const handleDiscover = async () => {
    setDiscovering(true);
    try {
      const discovery = await api.discoverConfig();
      setDiscoveredDirs(discovery.dirs);
    } finally {
      setDiscovering(false);
    }
  };

  const handleAddDiscoveredDir = async (path: string) => {
    const currentDirs = (config.scan_dirs || []).map((d) => d.path);
    if (!currentDirs.includes(path)) currentDirs.push(path);
    await api.updateConfig({ scan_dirs: currentDirs });
    queryClient.invalidateQueries({ queryKey: ["config"] });
    setDiscoveredDirs(null);
  };

  const handleDetectEmails = async () => {
    setDetectingEmails(true);
    try {
      const discovery = await api.discoverConfig();
      const currentTags = { ...config.email_tags };
      for (const e of discovery.emails) {
        if (!currentTags[e.email]) {
          currentTags[e.email] = e.current_tag || e.suggested_tag;
        }
      }
      await api.updateConfig({ email_tags: currentTags });
      queryClient.invalidateQueries({ queryKey: ["config"] });
    } finally {
      setDetectingEmails(false);
    }
  };

  const handleEmailTagChange = async (email: string, tag: string) => {
    const currentTags = { ...config.email_tags, [email]: tag };
    await api.updateConfig({ email_tags: currentTags });
    queryClient.invalidateQueries({ queryKey: ["config"] });
  };

  const integrationItems = [
    { key: "git", label: "Git", desc: "Local git repositories", configured: config.integrations?.git?.enabled },
    { key: "github", label: "GitHub", desc: "Pull requests and issues", configured: config.integrations?.github?.configured },
    { key: "linear", label: "Linear", desc: "Issue tracking", configured: config.integrations?.linear?.configured },
    { key: "claude_code", label: "Claude Code", desc: "AI coding sessions", configured: config.integrations?.claude_code?.configured },
  ];

  const dirsToShow = discoveredDirs || config.scan_dirs;

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Setup</h2>
        <p className="subtitle">Configure your sources, directories, and identities</p>
      </header>

      {showWelcome && (
        <div className="setup-welcome">
          <h3>Welcome to Folio</h3>
          <p>Let's get your career tracker set up. Configure your scan directories and git identities below, then sync to start tracking.</p>
        </div>
      )}

      <div className="setup-grid">
        <div className="card">
          <h3>Scan Directories</h3>
          <p className="setup-hint">Directories where folio looks for git repositories</p>
          <div>
            {!discoveredDirs && config.scan_dirs.length === 0 && (
              <p className="setup-empty">No scan directories configured</p>
            )}
            {!discoveredDirs &&
              config.scan_dirs.map((d) => (
                <div key={d.path} className="setup-dir-item">
                  <div className="setup-dir-info">
                    <span className="setup-dir-path">{d.path}</span>
                    <span className="setup-dir-meta">{d.exists ? `${d.repo_count} repos` : "directory not found"}</span>
                  </div>
                  <button className="setup-remove-btn" onClick={() => handleRemoveDir(d.path)} title="Remove">&times;</button>
                </div>
              ))}
            {discoveredDirs &&
              discoveredDirs.map((d) => (
                <div key={d.path} className={`setup-dir-item ${d.already_configured ? "" : "setup-dir-suggested"}`}>
                  <div className="setup-dir-info">
                    <span className="setup-dir-path">{d.path}</span>
                    <span className="setup-dir-meta">
                      {d.repo_count} repos ({d.repo_names.slice(0, 3).join(", ")}
                      {d.repo_names.length > 3 ? "..." : ""})
                    </span>
                  </div>
                  {d.already_configured ? (
                    <button className="setup-remove-btn" onClick={() => handleRemoveDir(d.path)}>&times;</button>
                  ) : (
                    <button className="btn btn-secondary" style={{ padding: "4px 12px", fontSize: 12 }} onClick={() => handleAddDiscoveredDir(d.path)}>
                      Add
                    </button>
                  )}
                </div>
              ))}
          </div>
          <div className="setup-add-row">
            <input type="text" placeholder="/path/to/code" className="setup-input" value={newDir} onChange={(e) => setNewDir(e.target.value)} />
            <button className="btn btn-secondary" onClick={handleAddDir}>Add</button>
          </div>
          <button className="btn btn-secondary" onClick={handleDiscover} disabled={discovering} style={{ marginTop: 8, width: "100%" }}>
            {discovering ? "Discovering..." : "Discover Directories"}
          </button>
        </div>

        <div className="card">
          <h3>Git Identities</h3>
          <p className="setup-hint">Map your git emails to work or personal tags</p>
          <div>
            {Object.entries(config.email_tags).length === 0 ? (
              <p className="setup-empty">No email identities mapped. Click "Detect Emails" to scan your repos.</p>
            ) : (
              Object.entries(config.email_tags).map(([email, tag]) => (
                <div key={email} className="setup-email-item">
                  <span className="setup-email-addr">{email}</span>
                  <select className="select-input setup-tag-select" value={tag} onChange={(e) => handleEmailTagChange(email, e.target.value)}>
                    <option value="personal">Personal</option>
                    <option value="work">Work</option>
                  </select>
                </div>
              ))
            )}
          </div>
          <button className="btn btn-secondary" onClick={handleDetectEmails} disabled={detectingEmails} style={{ marginTop: 8, width: "100%" }}>
            {detectingEmails ? "Detecting..." : "Detect Emails from Repos"}
          </button>
        </div>
      </div>

      <div className="card" style={{ marginTop: 20 }}>
        <h3>Integrations</h3>
        <div>
          {integrationItems.map((item) => (
            <div key={item.key} className="setup-integration-item">
              <div className={`setup-integration-status ${item.configured ? "configured" : "not-configured"}`} />
              <div className="setup-integration-info">
                <span className="setup-integration-label">{item.label}</span>
                <span className="setup-integration-desc">{item.desc}</span>
              </div>
              <span className="setup-integration-badge">{item.configured ? "Configured" : "Not configured"}</span>
            </div>
          ))}
        </div>
      </div>

      <div style={{ marginTop: 20, display: "flex", gap: 12 }}>
        <button className="btn btn-primary" onClick={() => setSyncStatus('Run "folio sync" in your terminal, then refresh this page to see results.')}>
          Sync Now
        </button>
        {syncStatus && <span style={{ color: "var(--text-muted)", fontSize: 13, lineHeight: "38px" }}>{syncStatus}</span>}
      </div>
    </section>
  );
}
