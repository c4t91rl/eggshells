import React from 'react';
import { SignatureAlgorithm } from '../types';

interface Props {
  algorithm: SignatureAlgorithm;
  className?: string;
}

const VerificationBadge: React.FC<Props> = ({ algorithm, className = '' }) => {
  const getConfig = () => {
    switch (algorithm) {
      case 'HybridEd25519MlDsa65':
        return {
          label: 'Hybrid PQ',
          icon: '🔐',
          className: 'quantum-badge',
          tooltip: 'Hybrid Ed25519 + ML-DSA-65 (Post-Quantum Safe)',
        };
      case 'MlDsa65':
        return {
          label: 'ML-DSA-65',
          icon: '🛡️',
          className: 'quantum-badge',
          tooltip: 'ML-DSA-65 / Dilithium3 (Post-Quantum)',
        };
      case 'Ed25519':
        return {
          label: 'Ed25519',
          icon: '🔑',
          className: 'status-info',
          tooltip: 'Ed25519 (Classical)',
        };
    }
  };

  const config = getConfig();

  return (
    <span
      className={`${config.className} ${className}`}
      title={config.tooltip}
    >
      {config.icon} {config.label}
    </span>
  );
};

export default VerificationBadge;