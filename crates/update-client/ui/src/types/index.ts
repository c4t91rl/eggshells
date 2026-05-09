export interface RegisteredServer {
  url: string;
  publisher: PublisherIdentity;
  enabled: boolean;
  last_checked: string | null;
  trust_level: TrustLevel;
}

export interface PublisherIdentity {
  id: string;
  name: string;
  description: string | null;
  server_url: string;
  algorithm: SignatureAlgorithm;
  ed25519_public_key: string | null;
  ml_dsa_public_key: string | null;
  key_id: string;
  created_at: string;
}

export type SignatureAlgorithm = 'Ed25519' | 'MlDsa65' | 'HybridEd25519MlDsa65';
export type TrustLevel = 'Pinned' | 'TrustOnFirstUse' | 'Verified' | 'Untrusted';
export type HashAlgorithm = 'Sha3_256' | 'Sha3_512' | 'Blake3';

export interface SignedManifest {
  manifest: UpdateManifest;
  signatures: ManifestSignature[];
}

export interface UpdateManifest {
  package_name: string;
  version: string;
  previous_version: string | null;
  timestamp: string;
  expires: string | null;
  files: FileEntry[];
  minimum_client_version: string | null;
  release_notes: string | null;
  publisher_id: string;
}

export interface FileEntry {
  path: string;
  size: number;
  hash_algorithm: HashAlgorithm;
  hash: string;
  download_url: string;
}

export interface ManifestSignature {
  algorithm: SignatureAlgorithm;
  publisher_id: string;
  key_id: string;
  signature: string;
  signed_at: string;
}

export interface VerificationResult {
  is_valid: boolean;
  checks: VerificationCheck[];
  warnings: string[];
  errors: string[];
}

export interface VerificationCheck {
  name: string;
  passed: boolean;
  details: string;
}

export interface AvailableUpdate {
  manifest: SignedManifest;
  verification: VerificationResult;
  publisher_name: string;
  server_url: string;
}

export interface IntegrityReport {
  checks: IntegrityCheck[];
  overall_status: IntegrityStatus;
  timestamp: string;
}

export interface IntegrityCheck {
  component: string;
  status: IntegrityStatus;
  details: string;
}

export type IntegrityStatus = 'Ok' | 'Warning' | 'Compromised' | 'Unknown';

export interface DownloadProgress {
  file_name: string;
  bytes_downloaded: number;
  total_bytes: number;
  percentage: number;
  speed_bytes_per_sec: number;
  status: DownloadStatus;
}

export type DownloadStatus = 'Pending' | 'Downloading' | 'Verifying' | 'Complete' | { Failed: string };

export interface SecurityInfo {
  supported_algorithms: AlgorithmInfo[];
  hash_algorithms: string[];
  tls_version: string;
}

export interface AlgorithmInfo {
  name: string;
  algorithm_type: string;
  key_size: string;
  signature_size: string;
  security_level: string;
  quantum_safe: boolean;
}