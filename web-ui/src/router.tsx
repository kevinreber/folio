import { createRouter, createRoute, createRootRoute } from '@tanstack/react-router'
import { RootLayout } from './routes/__root'
import { DashboardPage } from './routes/index'
import { ActivitiesPage } from './routes/activities'
import { ActivityDetailPage } from './routes/activities_.$id'
import { SearchPage } from './routes/search'
import { CapturePage } from './routes/capture'
import { WhatsNewPage } from './routes/whatsnew'

const rootRoute = createRootRoute({
  component: RootLayout,
})

const dashboardRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: DashboardPage,
})

const activitiesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/activities',
  component: ActivitiesPage,
})

const activityDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/activities/$id',
  component: ActivityDetailPage,
})

const searchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/search',
  component: SearchPage,
})

const captureRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/capture',
  component: CapturePage,
})

const whatsNewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/whatsnew',
  component: WhatsNewPage,
})

const routeTree = rootRoute.addChildren([
  dashboardRoute,
  activitiesRoute,
  activityDetailRoute,
  searchRoute,
  captureRoute,
  whatsNewRoute,
])

export const router = createRouter({ routeTree })
