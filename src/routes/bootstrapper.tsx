import { createFileRoute } from '@tanstack/react-router'
import Bootstrapper from '../pages/Bootstrapper'

export const Route = createFileRoute('/bootstrapper')({
  component: Bootstrapper,
})
