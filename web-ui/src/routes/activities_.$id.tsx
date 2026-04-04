import { useNavigate, useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { api } from '../lib/api'
import type { Activity } from '../lib/api'
import { ActivityModal } from '../components/ActivityModal'

export function ActivityDetailPage() {
  const { id } = useParams({ strict: false }) as { id: string }
  const navigate = useNavigate()
  const [activity, setActivity] = useState<Activity | null>(null)

  const load = () => {
    api.getActivity(id).then(setActivity).catch(() => navigate({ to: '/activities' }))
  }

  useEffect(() => {
    load()
  }, [id])

  if (!activity) return null

  return (
    <section className="view active">
      <div className="modal" style={{ position: 'relative', inset: 'auto', display: 'flex', justifyContent: 'center', paddingTop: 40 }}>
        <div className="modal-content" style={{ animation: 'none' }}>
          <ActivityModal
            activity={activity}
            onClose={() => navigate({ to: '/activities' })}
            onUpdated={load}
          />
        </div>
      </div>
    </section>
  )
}
