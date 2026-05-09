import React from 'react';
import { useAppStore } from '../store/appStore';
import {
  HiOutlineHome,
  HiOutlineServer,
  HiOutlineArrowDown,
  HiOutlineShieldCheck,
  HiOutlineCog6Tooth,
  HiOutlineDocumentText,
} from 'react-icons/hi2';
import { IoShieldCheckmark } from 'react-icons/io5';

type Page = 'dashboard' | 'servers' | 'updates' | 'security' | 'settings' | 'logs';

const Sidebar: React.FC = () => {
  const { currentPage, setPage, availableUpdates, integrityReport } = useAppStore();

  const menuItems: { id: Page; label: string; icon: React.ReactNode; badge?: number }[] = [
    { id: 'dashboard', label: 'Dashboard', icon: <HiOutlineHome size={20} /> },
    { id: 'servers', label: 'Servers', icon: <HiOutlineServer size={20} /> },
    {
      id: 'updates',
      label: 'Updates',
      icon: <HiOutlineArrowDown size={20} />,
      badge: availableUpdates.length || undefined,
    },
    { id: 'security', label: 'Security', icon: <HiOutlineShieldCheck size={20} /> },
    { id: 'settings', label: 'Settings', icon: <HiOutlineCog6Tooth size={20} /> },
    { id: 'logs', label: 'Activity Log', icon: <HiOutlineDocumentText size={20} /> },
  ];

  return (
    <aside className="w-64 h-screen bg-dark-900/80 backdrop-blur-xl border-r border-dark-700/50 flex flex-col">
      {/* Logo */}
      <div className="p-6 flex items-center gap-3">
        <div className="relative">
          <IoShieldCheckmark size={32} className="text-quantum-400 animate-shield-glow" />
          <div className="absolute -top-1 -right-1 w-3 h-3 bg-green-400 rounded-full border-2 border-dark-900" />
        </div>
        <div>
          <h1 className="text-lg font-bold gradient-text">KryptoUpdate</h1>
          <p className="text-xs text-dark-500">Post-Quantum Secure</p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 space-y-1">
        {menuItems.map((item) => (
          <button
            key={item.id}
            onClick={() => setPage(item.id)}
            className={`w-full ${
              currentPage === item.id ? 'sidebar-item-active' : 'sidebar-item'
            }`}
          >
            {item.icon}
            <span className="flex-1 text-left text-sm font-medium">{item.label}</span>
            {item.badge && (
              <span className="bg-primary-600 text-white text-xs px-2 py-0.5 rounded-full">
                {item.badge}
              </span>
            )}
          </button>
        ))}
      </nav>

      {/* Status footer */}
      <div className="p-4 border-t border-dark-700/50">
        <div className="glass-card p-3 rounded-xl">
          <div className="flex items-center gap-2 mb-2">
            <div
              className={`w-2 h-2 rounded-full ${
                integrityReport?.overall_status === 'Ok'
                  ? 'bg-green-400'
                  : integrityReport?.overall_status === 'Warning'
                  ? 'bg-yellow-400'
                  : 'bg-red-400'
              } animate-pulse`}
            />
            <span className="text-xs text-dark-300">System Integrity</span>
          </div>
          <p className="text-xs text-dark-500 font-mono">
            {integrityReport?.overall_status ?? 'Checking...'}
          </p>
        </div>
      </div>
    </aside>
  );
};

export default Sidebar;