import { createRoot } from 'react-dom/client';
import TransferApp from './TransferApp';
import './styles.css';

document.body.dataset.window = 'transfer';
createRoot(document.getElementById('root')!).render(<TransferApp />);
