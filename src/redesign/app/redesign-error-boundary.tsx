/*
 * Global error boundary for the redesign tree (consolidated-spec.md §13). A render crash lands on a
 * styled fallback with a reload action instead of a white screen; the error itself goes to the
 * console as the developer trail.
 */
import { Component, type ErrorInfo, type ReactNode } from 'react'
import { FidelityPanel } from '../shared/components/fidelity-panel'
import { FidelityButton } from '../shared/components/fidelity-button'
import { commonText } from '../i18n/common-text'

type Props = { children: ReactNode }
type State = { error: Error | null }

export class RedesignErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[redesign] render error', error, info.componentStack)
  }

  render() {
    if (!this.state.error) return this.props.children
    return (
      <div className="flex min-h-screen items-center justify-center bg-[var(--mk-surface)] p-6">
        <FidelityPanel className="flex max-w-md flex-col items-center gap-4 p-8 text-center">
          <h1 className="text-lg font-semibold text-[var(--mk-text)]">
            {commonText.somethingWentWrong()}
          </h1>
          <p className="break-all text-sm text-[var(--mk-text-muted)]">
            {this.state.error.message}
          </p>
          <FidelityButton onClick={() => window.location.reload()}>
            {commonText.reload()}
          </FidelityButton>
        </FidelityPanel>
      </div>
    )
  }
}
