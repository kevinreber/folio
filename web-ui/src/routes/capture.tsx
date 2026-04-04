import { createRoute } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { rootRoute } from "./__root";
import { api } from "../api/client";

export const captureRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/capture",
  component: CapturePage,
});

function CapturePage() {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [impact, setImpact] = useState("");
  const [importance, setImportance] = useState("medium");
  const [project, setProject] = useState("");
  const [employer, setEmployer] = useState("");
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;

    try {
      await api.createActivity({
        title: title.trim(),
        description: description.trim() || undefined,
        impact: impact.trim() || undefined,
        importance,
        project: project.trim() || undefined,
        employer: employer.trim() || undefined,
      });
      setTitle("");
      setDescription("");
      setImpact("");
      setImportance("medium");
      setProject("");
      setEmployer("");
      setError(false);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
      queryClient.invalidateQueries({ queryKey: ["activities"] });
      queryClient.invalidateQueries({ queryKey: ["stats"] });
    } catch {
      setError(true);
    }
  };

  const handleReset = () => {
    setTitle("");
    setDescription("");
    setImpact("");
    setImportance("medium");
    setProject("");
    setEmployer("");
    setSuccess(false);
    setError(false);
  };

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Capture Activity</h2>
        <p className="subtitle">Record what you accomplished while it's fresh</p>
      </header>
      <form className="form-card" onSubmit={handleSubmit}>
        <div className="form-group">
          <label>What did you do?</label>
          <input type="text" placeholder="e.g. Implemented new auth system with OAuth2" required value={title} onChange={(e) => setTitle(e.target.value)} />
        </div>
        <div className="form-group">
          <label>Description <span className="optional">(optional)</span></label>
          <textarea rows={3} placeholder="Add more details about what you did..." value={description} onChange={(e) => setDescription(e.target.value)} />
        </div>
        <div className="form-row">
          <div className="form-group">
            <label>Impact <span className="optional">(optional)</span></label>
            <input type="text" placeholder="e.g. Reduced login time by 50%" value={impact} onChange={(e) => setImpact(e.target.value)} />
          </div>
          <div className="form-group">
            <label>Importance</label>
            <select className="select-input" value={importance} onChange={(e) => setImportance(e.target.value)}>
              <option value="medium">Medium</option>
              <option value="high">High</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>
        <div className="form-row">
          <div className="form-group">
            <label>Project <span className="optional">(optional)</span></label>
            <input type="text" placeholder="e.g. auth-service" value={project} onChange={(e) => setProject(e.target.value)} />
          </div>
          <div className="form-group">
            <label>Employer <span className="optional">(optional)</span></label>
            <input type="text" placeholder="e.g. Acme Corp" value={employer} onChange={(e) => setEmployer(e.target.value)} />
          </div>
        </div>
        <div className="form-actions">
          <button type="submit" className="btn btn-primary">Capture Activity</button>
          <button type="button" className="btn btn-secondary" onClick={handleReset}>Clear</button>
        </div>
        {success && <div className="success-message">Activity captured successfully!</div>}
        {error && <div className="error-message">Failed to capture activity. Is the API server running?</div>}
      </form>
    </section>
  );
}
