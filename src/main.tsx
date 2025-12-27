import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider } from '@tanstack/react-router'
import Router from '@utils/router.ts'
import './assets/style.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
      <RouterProvider router={Router} />
  </StrictMode>,
)
