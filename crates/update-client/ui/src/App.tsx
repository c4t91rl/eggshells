import React, { useEffect } from 'react';
import { useAppStore } from './store/appStore';
import { api } from './utils/tauriApi';
import Layout from './components/Layout';
import Dashboard from './components/Dashboard';
import ServerManager from './components/ServerManager';
import UpdateList from './components/UpdateList';
import SecurityStatus from './components/SecurityStatus';
import SettingsPanel from './components/SettingsPanel';
import LogViewer from './components/LogViewer';

const App: React.FC = () => {
  const { currentPage, setServers, addLog, setSecurityInfo, setIntegrityReport } = useAppStore();

  useEffect(() => {
    const init = async () => {
      try {
        addLog('info', 'Initializing KryptoUpdate...');

        // Load servers
        const servers = await api.getServers();
        setServers(servers);
        addLog('info', `Loaded ${servers.length} registered server(s)`);

        // Load security info
        const secInfo = await api.getSecurityInfo();
        setSecurityInfo(secInfo);
        addLog('info', 'Security configuration loaded');

        // Run integrity check
        const report = await api.getIntegrityReport();
        setIntegrityReport(report);
        addLog(
          report.overall_status === 'Ok' ? 'success' : 'warn',
          `Integrity check: ${report.overall_status}`
        );

        addLog('success', 'KryptoUpdate initialized successfully');
      } catch (err) {
        addLog('error', `Initialization failed: ${err}`);
      }
    };

    init();
  }, []);

  const renderPage = () => {
    switch (currentPage) {
      case 'dashboard': return <Dashboard />;
      case 'servers': return <ServerManager />;
      case 'updates': return <UpdateList />;
      case 'security': return <SecurityStatus />;
      case 'settings': return <SettingsPanel />;
      case 'logs': return <LogViewer />;
      default: return <Dashboard />;
    }
  };

  return (
    <Layout>
      {renderPage()}
    </Layout>
  );
};

export default App;