import { rootRoute } from "./routes/__root";
import { dashboardRoute } from "./routes/dashboard";
import { activitiesRoute } from "./routes/activities";
import { searchRoute } from "./routes/search";
import { timelineRoute } from "./routes/timeline";
import { captureRoute } from "./routes/capture";
import { accomplishmentsRoute } from "./routes/accomplishments";
import { claudeRoute } from "./routes/claude";
import { setupRoute } from "./routes/setup";
import { whatsnewRoute } from "./routes/whatsnew";

export const routeTree = rootRoute.addChildren([
  dashboardRoute,
  activitiesRoute,
  searchRoute,
  timelineRoute,
  captureRoute,
  accomplishmentsRoute,
  claudeRoute,
  setupRoute,
  whatsnewRoute,
]);
