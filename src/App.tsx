import { useEffect, useState } from 'react';

import { AppShell } from './layouts/AppShell';
import { DevGalleryPage } from './pages/DevGalleryPage';

function App() {
  const [route, setRoute] = useState(() => window.location.hash);

  useEffect(() => {
    const handleHashChange = () => setRoute(window.location.hash);
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  return (
    <AppShell>
      {import.meta.env.DEV && route === '#/dev/gallery' ? <DevGalleryPage /> : null}
    </AppShell>
  );
}

export default App;
