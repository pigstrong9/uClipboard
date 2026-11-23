import { createRoot } from 'react-dom/client';
import ScreenshotApp from './ScreenshotApp';
import './styles.css';

document.body.dataset.window = 'screenshot';
createRoot(document.getElementById('root')!).render(<ScreenshotApp />);
