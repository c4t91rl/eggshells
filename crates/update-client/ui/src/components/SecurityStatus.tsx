import React, { useEffect } from 'react';
import { useAppStore } from '../store/appStore';
import { api } from '../utils/tauriApi';
import {
  HiOutlineShieldCheck,
  HiOutlineShieldExclamation,
  HiOutlineCpuChip,
  HiOutlineFingerPrint,
  HiOutlineLockClosed,
  HiOutlineArrowPath,
} from 'react-icons/hi2';
import { motion } from 'framer-motion';

const SecurityStatus: React.FC = () => {
  const {
    integrityReport,
    setIntegrityReport,
    securityInfo,
    addLog,
  } = useAppStore();

  const handleRefreshIntegrity = async () => {
    try {
      const report = await api.getIntegrityReport();
      setIntegrityReport(report);
      addLog(
        report.overall_status === 'Ok' ? 'success' : 'warn',
        `Integrity re-check: ${report.overall_status}`
      );
    } catch (err) {
      addLog('error', `Integrity check failed: ${err}`);
    }
  };

  const statusColors = {
    Ok: { bg: 'bg-green-500/15', text: 'text-green-400', border: 'border-green-500/20' },
    Warning: { bg: 'bg-yellow-500/15', text: 'text-yellow-400', border: 'border-yellow-500/20' },
    Compromised: { bg: 'bg-red-500/15', text: 'text-red-400', border: 'border-red-500/20' },
    Unknown: { bg: 'bg-gray-500/15', text: 'text-gray-400', border: 'border-gray-500/20' },
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Security Status</h2>
          <p className="text-dark-400 text-sm mt-1">
            System integrity, cryptographic capabilities, and threat analysis
          </p>
        </div>
        <button onClick={handleRefreshIntegrity} className="btn-secondary flex items-center gap-2">
          <HiOutlineArrowPath size={18} />
          Re-check Integrity
        </button>
      </div>

      {/* Overall Status */}
      {integrityReport && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className={`glass-card p-6 border-2 ${
            statusColors[integrityReport.overall_status]?.border
          }`}
        >
          <div className="flex items-center gap-4">
            <div className={`p-3 rounded-2xl ${statusColors[integrityReport.overall_status]?.bg}`}>
              {integrityReport.overall_status === 'Ok' ? (
                <HiOutlineShieldCheck size={40} className="text-green-400" />
              ) : (
                <HiOutlineShieldExclamation size={40} className="text-yellow-400" />
              )}
            </div>
            <div>
              <h3 className={`text-2xl font-bold ${
                statusColors[integrityReport.overall_status]?.text
              }`}>
                System {integrityReport.overall_status}
              </h3>
              <p className="text-dark-400 text-sm">
                Last checked: {new Date(integrityReport.timestamp).toLocaleString()}
              </p>
            </div>
          </div>
        </motion.div>
      )}

      {/* Integrity Checks */}
      {integrityReport && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="glass-card p-6"
        >
          <h3 className="section-title flex items-center gap-2 mb-4">
            <HiOutlineFingerPrint size={20} className="text-primary-400" />
            Integrity Checks
          </h3>

          <div className="space-y-3">
            {integrityReport.checks.map((check, idx) => {
              const colors = statusColors[check.status];
              return (
                <div
                  key={idx}
                  className={`flex items-center justify-between p-4 rounded-xl border ${colors?.border} ${colors?.bg}`}
                >
                  <div className="flex items-center gap-3">
                    <span className={`text-lg ${colors?.text}`}>
                      {check.status === 'Ok' ? '✓' : check.status === 'Warning' ? '⚠' : '✗'}
                    </span>
                    <div>
                      <h4 className="font-medium text-dark-100">{check.component}</h4>
                      <p className="text-xs text-dark-400 mt-0.5 font-mono">{check.details}</p>
                    </div>
                  </div>
                  <span className={`text-sm font-medium ${colors?.text}`}>{check.status}</span>
                </div>
              );
            })}
          </div>
        </motion.div>
      )}

      {/* Cryptographic Capabilities */}
      {securityInfo && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="glass-card p-6"
        >
          <h3 className="section-title flex items-center gap-2 mb-4">
            <HiOutlineLockClosed size={20} className="text-quantum-400" />
            Cryptographic Capabilities
          </h3>

          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-dark-700/50">
                  <th className="text-left py-3 px-4 text-dark-400 font-medium">Algorithm</th>
                  <th className="text-left py-3 px-4 text-dark-400 font-medium">Type</th>
                  <th className="text-left py-3 px-4 text-dark-400 font-medium">Key Size</th>
                  <th className="text-left py-3 px-4 text-dark-400 font-medium">Security Level</th>
                  <th className="text-left py-3 px-4 text-dark-400 font-medium">Quantum Safe</th>
                </tr>
              </thead>
              <tbody>
                {securityInfo.supported_algorithms.map((algo) => (
                  <tr key={algo.name} className="border-b border-dark-800/50 hover:bg-dark-800/30">
                    <td className="py-3 px-4 font-semibold text-dark-100">{algo.name}</td>
                    <td className="py-3 px-4 text-dark-300 font-mono text-xs">{algo.algorithm_type}</td>
                    <td className="py-3 px-4 text-dark-300 font-mono text-xs">{algo.key_size}</td>
                    <td className="py-3 px-4 text-dark-300 font-mono text-xs">{algo.security_level}</td>
                    <td className="py-3 px-4">
                      {algo.quantum_safe ? (
                        <span className="quantum-badge">🛡️ Yes</span>
                      ) : (
                        <span className="status-warning">⚠️ No</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="mt-4 p-4 bg-dark-800/30 rounded-xl border border-dark-700/30">
            <h4 className="text-sm font-semibold text-dark-300 mb-2 flex items-center gap-2">
              <HiOutlineCpuChip size={16} />
              Hash Algorithms
            </h4>
            <div className="flex gap-2 flex-wrap">
              {securityInfo.hash_algorithms.map((hash) => (
                <span key={hash} className="status-info">
                  {hash}
                </span>
              ))}
            </div>
          </div>
        </motion.div>
      )}

      {/* Threat Model Summary */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="glass-card p-6"
      >
        <h3 className="section-title mb-4">Threat Model</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {[
            {
              threat: 'Man-in-the-Middle (MITM)',
              mitigation: 'TLS + Digital Signatures',
              status: 'Mitigated',
            },
            {
              threat: 'Package Tampering',
              mitigation: 'Cryptographic hash verification (BLAKE3/SHA3)',
              status: 'Mitigated',
            },
            {
              threat: 'Signature Forgery',
              mitigation: 'Hybrid PQ signatures (Ed25519 + ML-DSA-65)',
              status: 'Mitigated',
            },
            {
              threat: 'Downgrade Attack',
              mitigation: 'Version chain tracking & minimum version',
              status: 'Mitigated',
            },
            {
              threat: 'Key Compromise',
              mitigation: 'Key revocation, multiple signatures',
              status: 'Partially Mitigated',
            },
            {
              threat: 'Quantum Attack',
              mitigation: 'Post-quantum ML-DSA-65 (Dilithium3)',
              status: 'Mitigated',
            },
          ].map((item, idx) => (
            <div key={idx} className="bg-dark-800/50 p-4 rounded-xl border border-dark-700/30">
              <h4 className="font-semibold text-dark-200 text-sm mb-1">{item.threat}</h4>
              <p className="text-xs text-dark-400 mb-2">{item.mitigation}</p>
              <span className={
                item.status === 'Mitigated' ? 'status-ok' : 'status-warning'
              }>
                {item.status}
              </span>
            </div>
          ))}
        </div>
      </motion.div>
    </div>
  );
};

export default SecurityStatus;