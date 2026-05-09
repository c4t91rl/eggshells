import React, { useEffect, useState } from 'react';
import { useAppStore } from '../store/appStore';
import { api } from '../utils/tauriApi';
import {
  HiOutlineServer,
  HiOutlineArrowDown,
  HiOutlineShieldCheck,
  HiOutlineExclamationTriangle,
  HiOutlineCheckCircle,
  HiOutlineArrowPath,
} from 'react-icons/hi2';
import { motion } from 'framer-motion';

const Dashboard: React.FC = () => {
  const {
    servers,
    availableUpdates,
    setAvailableUpdates,
    isCheckingUpdates,
    setCheckingUpdates,
    integrityReport,
    securityInfo,
    addLog,
  } = useAppStore();

  const handleCheckUpdates = async () => {
    setCheckingUpdates(true);
    addLog('info', 'Checking for updates across all servers...');
    try {
      const updates = await api.checkAllUpdates();
      setAvailableUpdates(updates);
      addLog('success', `Found ${updates.length} available update(s)`);
    } catch (err) {
      addLog('error', `Update check failed: ${err}`);
    } finally {
      setCheckingUpdates(false);
    }
  };

  const statCards = [
    {
      title: 'Registered Servers',
      value: servers.length,
      icon: <HiOutlineServer size={24} />,
      color: 'text-blue-400',
      bg: 'bg-blue-500/10',
    },
    {
      title: 'Available Updates',
      value: availableUpdates.length,
      icon: <HiOutlineArrowDown size={24} />,
      color: 'text-primary-400',
      bg: 'bg-primary-500/10',
    },
    {
      title: 'Verified Signatures',
      value: availableUpdates.filter((u) => u.verification.is_valid).length,
      icon: <HiOutlineCheckCircle size={24} />,
      color: 'text-green-400',
      bg: 'bg-green-500/10',
    },
    {
      title: 'Security Warnings',
      value:
        (integrityReport?.checks.filter((c) => c.status !== 'Ok').length ?? 0) +
        availableUpdates.filter((u) => !u.verification.is_valid).length,
      icon: <HiOutlineExclamationTriangle size={24} />,
      color: 'text-yellow-400',
      bg: 'bg-yellow-500/10',
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Dashboard</h2>
          <p className="text-dark-400 text-sm mt-1">
            Post-quantum secure software update management
          </p>
        </div>
        <button
          onClick={handleCheckUpdates}
          disabled={isCheckingUpdates}
          className="btn-primary flex items-center gap-2"
        >
          <HiOutlineArrowPath
            size={18}
            className={isCheckingUpdates ? 'animate-spin' : ''}
          />
          {isCheckingUpdates ? 'Checking...' : 'Check for Updates'}
        </button>
      </div>

      {/* Stat Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {statCards.map((stat, index) => (
          <motion.div
            key={stat.title}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
            className="glass-card p-5"
          >
            <div className="flex items-center justify-between mb-3">
              <div className={`${stat.bg} p-2.5 rounded-xl ${stat.color}`}>
                {stat.icon}
              </div>
            </div>
            <div className="text-3xl font-bold text-dark-50">{stat.value}</div>
            <div className="text-sm text-dark-400 mt-1">{stat.title}</div>
          </motion.div>
        ))}
      </div>

      {/* Cryptography Info */}
      {securityInfo && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="glass-card p-6"
        >
          <h3 className="section-title flex items-center gap-2">
            <HiOutlineShieldCheck size={20} className="text-quantum-400" />
            Supported Cryptography
          </h3>
          <p className="section-subtitle mb-4">
            Post-quantum and classical algorithms available for signature verification
          </p>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {securityInfo.supported_algorithms.map((algo) => (
              <div
                key={algo.name}
                className="bg-dark-800/50 rounded-xl p-4 border border-dark-700/50"
              >
                <div className="flex items-center justify-between mb-3">
                  <h4 className="font-semibold text-dark-100 text-sm">{algo.name}</h4>
                  {algo.quantum_safe ? (
                    <span className="quantum-badge">
                      🛡️ Quantum Safe
                    </span>
                  ) : (
                    <span className="status-info">Classical</span>
                  )}
                </div>
                <div className="space-y-1.5 text-xs text-dark-400">
                  <div className="flex justify-between">
                    <span>Type:</span>
                    <span className="text-dark-300 font-mono">{algo.algorithm_type}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Key Size:</span>
                    <span className="text-dark-300 font-mono">{algo.key_size}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Security:</span>
                    <span className="text-dark-300 font-mono">{algo.security_level}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      )}

      {/* Recent Updates */}
      {availableUpdates.length > 0 && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5 }}
          className="glass-card p-6"
        >
          <h3 className="section-title">Recent Updates</h3>
          <p className="section-subtitle mb-4">Latest available updates from registered servers</p>

          <div className="space-y-3">
            {availableUpdates.slice(0, 5).map((update, idx) => (
              <div
                key={idx}
                className="flex items-center justify-between bg-dark-800/50 rounded-xl p-4 border border-dark-700/50"
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`w-10 h-10 rounded-xl flex items-center justify-center ${
                      update.verification.is_valid
                        ? 'bg-green-500/15 text-green-400'
                        : 'bg-red-500/15 text-red-400'
                    }`}
                  >
                    {update.verification.is_valid ? (
                      <HiOutlineCheckCircle size={24} />
                    ) : (
                      <HiOutlineExclamationTriangle size={24} />
                    )}
                  </div>
                  <div>
                    <h4 className="font-semibold text-dark-100">
                      {update.manifest.manifest.package_name}
                    </h4>
                    <p className="text-xs text-dark-400">
                      v{update.manifest.manifest.version} • {update.publisher_name}
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  {update.manifest.signatures.map((sig, sigIdx) => (
                    <span
                      key={sigIdx}
                      className={
                        sig.algorithm === 'HybridEd25519MlDsa65'
                          ? 'quantum-badge'
                          : sig.algorithm === 'MlDsa65'
                          ? 'quantum-badge'
                          : 'status-info'
                      }
                    >
                      {sig.algorithm === 'HybridEd25519MlDsa65'
                        ? '🔐 Hybrid PQ'
                        : sig.algorithm === 'MlDsa65'
                        ? '🛡️ ML-DSA'
                        : '🔑 Ed25519'}
                    </span>
                  ))}
                  <button className="btn-primary text-sm py-1.5">Install</button>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      )}
    </div>
  );
};

export default Dashboard;