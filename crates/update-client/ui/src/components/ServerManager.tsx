import React, { useState } from 'react';
import { useAppStore } from '../store/appStore';
import { api } from '../utils/tauriApi';
import {
  HiOutlinePlusCircle,
  HiOutlineTrash,
  HiOutlineServer,
  HiOutlineGlobeAlt,
  HiOutlineKey,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
} from 'react-icons/hi2';
import { motion, AnimatePresence } from 'framer-motion';
import { RegisteredServer, TrustLevel } from '../types';

const ServerManager: React.FC = () => {
  const { servers, setServers, addLog } = useAppStore();
  const [newServerUrl, setNewServerUrl] = useState('');
  const [isAdding, setIsAdding] = useState(false);
  const [showAddForm, setShowAddForm] = useState(false);
  const [selectedServer, setSelectedServer] = useState<RegisteredServer | null>(null);

  const handleAddServer = async () => {
    if (!newServerUrl.trim()) return;
    setIsAdding(true);
    try {
      const server = await api.addServer(newServerUrl.trim());
      const updated = await api.getServers();
      setServers(updated);
      setNewServerUrl('');
      setShowAddForm(false);
      addLog('success', `Server added: ${server.publisher.name} (${server.publisher.id})`);
    } catch (err) {
      addLog('error', `Failed to add server: ${err}`);
    } finally {
      setIsAdding(false);
    }
  };

  const handleRemoveServer = async (publisherId: string) => {
    try {
      await api.removeServer(publisherId);
      const updated = await api.getServers();
      setServers(updated);
      setSelectedServer(null);
      addLog('info', `Server removed: ${publisherId}`);
    } catch (err) {
      addLog('error', `Failed to remove server: ${err}`);
    }
  };

  const getTrustBadge = (level: TrustLevel) => {
    switch (level) {
      case 'Pinned': return <span className="status-ok">📌 Pinned</span>;
      case 'Verified': return <span className="status-ok">✅ Verified</span>;
      case 'TrustOnFirstUse': return <span className="status-warning">🤝 TOFU</span>;
      case 'Untrusted': return <span className="status-error">⚠️ Untrusted</span>;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Update Servers</h2>
          <p className="text-dark-400 text-sm mt-1">
            Manage trusted publishers and their signing keys
          </p>
        </div>
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          className="btn-primary flex items-center gap-2"
        >
          <HiOutlinePlusCircle size={18} />
          Add Server
        </button>
      </div>

      {/* Add Server Form */}
      <AnimatePresence>
        {showAddForm && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="glass-card p-6 overflow-hidden"
          >
            <h3 className="font-semibold text-dark-100 mb-4">Add Update Server</h3>
            <div className="flex gap-3">
              <input
                type="text"
                value={newServerUrl}
                onChange={(e) => setNewServerUrl(e.target.value)}
                placeholder="https://update-server.example.com"
                className="input-field flex-1"
                onKeyDown={(e) => e.key === 'Enter' && handleAddServer()}
              />
              <button
                onClick={handleAddServer}
                disabled={isAdding || !newServerUrl.trim()}
                className="btn-primary whitespace-nowrap"
              >
                {isAdding ? 'Discovering...' : 'Discover & Add'}
              </button>
            </div>
            <p className="text-xs text-dark-500 mt-2">
              The client will connect to the server, fetch its public keys, and register it
              using Trust On First Use (TOFU) model.
            </p>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Server List */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {servers.map((server, idx) => (
          <motion.div
            key={server.publisher.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: idx * 0.05 }}
            className={`glass-card-hover p-5 cursor-pointer ${
              selectedServer?.publisher.id === server.publisher.id
                ? 'ring-2 ring-primary-500/50'
                : ''
            }`}
            onClick={() => setSelectedServer(server)}
          >
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl bg-primary-500/10 flex items-center justify-center">
                  <HiOutlineServer size={20} className="text-primary-400" />
                </div>
                <div>
                  <h4 className="font-semibold text-dark-100">{server.publisher.name}</h4>
                  <p className="text-xs text-dark-400">{server.publisher.id}</p>
                </div>
              </div>
              {getTrustBadge(server.trust_level)}
            </div>

            <div className="space-y-2 text-sm">
              <div className="flex items-center gap-2 text-dark-400">
                <HiOutlineGlobeAlt size={14} />
                <span className="font-mono text-xs truncate">{server.url}</span>
              </div>
              <div className="flex items-center gap-2 text-dark-400">
                <HiOutlineKey size={14} />
                <span className="font-mono text-xs">
                  {server.publisher.algorithm === 'HybridEd25519MlDsa65'
                    ? '🔐 Hybrid (Ed25519 + ML-DSA-65)'
                    : server.publisher.algorithm === 'MlDsa65'
                    ? '🛡️ ML-DSA-65 (Dilithium3)'
                    : '🔑 Ed25519'}
                </span>
              </div>
              <div className="flex items-center gap-2 text-dark-400">
                <span className="text-xs">Key ID: </span>
                <code className="text-xs bg-dark-800 px-1.5 py-0.5 rounded font-mono text-dark-300">
                  {server.publisher.key_id}
                </code>
              </div>
            </div>

            <div className="flex items-center justify-between mt-4 pt-3 border-t border-dark-700/50">
              <span className="text-xs text-dark-500">
                {server.last_checked
                  ? `Last checked: ${new Date(server.last_checked).toLocaleString()}`
                  : 'Never checked'}
              </span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleRemoveServer(server.publisher.id);
                }}
                className="text-red-400 hover:text-red-300 p-1 rounded-lg hover:bg-red-500/10 transition-colors"
              >
                <HiOutlineTrash size={16} />
              </button>
            </div>
          </motion.div>
        ))}
      </div>

      {servers.length === 0 && (
        <div className="glass-card p-12 text-center">
          <HiOutlineServer size={48} className="text-dark-600 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-dark-300">No Servers Registered</h3>
          <p className="text-dark-500 text-sm mt-2">
            Add an update server to start receiving secure software updates.
          </p>
        </div>
      )}

      {/* Server Detail Panel */}
      <AnimatePresence>
        {selectedServer && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            className="glass-card p-6"
          >
            <h3 className="section-title mb-4">
              Server Details: {selectedServer.publisher.name}
            </h3>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Public Keys */}
              <div>
                <h4 className="text-sm font-semibold text-dark-300 mb-3">Public Keys</h4>
                {selectedServer.publisher.ed25519_public_key && (
                  <div className="mb-3">
                    <label className="text-xs text-dark-500 block mb-1">Ed25519 Public Key</label>
                    <code className="text-xs bg-dark-800 p-2 rounded-lg block font-mono text-dark-300 break-all">
                      {selectedServer.publisher.ed25519_public_key}
                    </code>
                  </div>
                )}
                {selectedServer.publisher.ml_dsa_public_key && (
                  <div>
                    <label className="text-xs text-dark-500 block mb-1">ML-DSA-65 Public Key</label>
                    <code className="text-xs bg-dark-800 p-2 rounded-lg block font-mono text-quantum-300 break-all max-h-20 overflow-y-auto">
                      {selectedServer.publisher.ml_dsa_public_key.substring(0, 120)}...
                    </code>
                    <span className="text-xs text-dark-500">
                      ({selectedServer.publisher.ml_dsa_public_key.length} chars)
                    </span>
                  </div>
                )}
              </div>

              {/* Trust Info */}
              <div>
                <h4 className="text-sm font-semibold text-dark-300 mb-3">Trust Information</h4>
                <div className="space-y-3">
                  <div className="flex justify-between text-sm">
                    <span className="text-dark-400">Trust Level</span>
                    {getTrustBadge(selectedServer.trust_level)}
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-dark-400">Algorithm</span>
                    <span className="text-dark-200 font-mono text-xs">
                      {selectedServer.publisher.algorithm}
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-dark-400">Registered</span>
                    <span className="text-dark-200 text-xs">
                      {new Date(selectedServer.publisher.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default ServerManager;