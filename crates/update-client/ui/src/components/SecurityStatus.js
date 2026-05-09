"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
var react_1 = require("react");
var appStore_1 = require("../store/appStore");
var tauriApi_1 = require("../utils/tauriApi");
var hi2_1 = require("react-icons/hi2");
var framer_motion_1 = require("framer-motion");
var SecurityStatus = function () {
    var _a, _b, _c;
    var _d = (0, appStore_1.useAppStore)(), integrityReport = _d.integrityReport, setIntegrityReport = _d.setIntegrityReport, securityInfo = _d.securityInfo, addLog = _d.addLog;
    var handleRefreshIntegrity = function () { return __awaiter(void 0, void 0, void 0, function () {
        var report, err_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4 /*yield*/, tauriApi_1.api.getIntegrityReport()];
                case 1:
                    report = _a.sent();
                    setIntegrityReport(report);
                    addLog(report.overall_status === 'Ok' ? 'success' : 'warn', "Integrity re-check: ".concat(report.overall_status));
                    return [3 /*break*/, 3];
                case 2:
                    err_1 = _a.sent();
                    addLog('error', "Integrity check failed: ".concat(err_1));
                    return [3 /*break*/, 3];
                case 3: return [2 /*return*/];
            }
        });
    }); };
    var statusColors = {
        Ok: { bg: 'bg-green-500/15', text: 'text-green-400', border: 'border-green-500/20' },
        Warning: { bg: 'bg-yellow-500/15', text: 'text-yellow-400', border: 'border-yellow-500/20' },
        Compromised: { bg: 'bg-red-500/15', text: 'text-red-400', border: 'border-red-500/20' },
        Unknown: { bg: 'bg-gray-500/15', text: 'text-gray-400', border: 'border-gray-500/20' },
    };
    return (<div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Security Status</h2>
          <p className="text-dark-400 text-sm mt-1">
            System integrity, cryptographic capabilities, and threat analysis
          </p>
        </div>
        <button onClick={handleRefreshIntegrity} className="btn-secondary flex items-center gap-2">
          <hi2_1.HiOutlineArrowPath size={18}/>
          Re-check Integrity
        </button>
      </div>

      {/* Overall Status */}
      {integrityReport && (<framer_motion_1.motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className={"glass-card p-6 border-2 ".concat((_a = statusColors[integrityReport.overall_status]) === null || _a === void 0 ? void 0 : _a.border)}>
          <div className="flex items-center gap-4">
            <div className={"p-3 rounded-2xl ".concat((_b = statusColors[integrityReport.overall_status]) === null || _b === void 0 ? void 0 : _b.bg)}>
              {integrityReport.overall_status === 'Ok' ? (<hi2_1.HiOutlineShieldCheck size={40} className="text-green-400"/>) : (<hi2_1.HiOutlineShieldExclamation size={40} className="text-yellow-400"/>)}
            </div>
            <div>
              <h3 className={"text-2xl font-bold ".concat((_c = statusColors[integrityReport.overall_status]) === null || _c === void 0 ? void 0 : _c.text)}>
                System {integrityReport.overall_status}
              </h3>
              <p className="text-dark-400 text-sm">
                Last checked: {new Date(integrityReport.timestamp).toLocaleString()}
              </p>
            </div>
          </div>
        </framer_motion_1.motion.div>)}

      {/* Integrity Checks */}
      {integrityReport && (<framer_motion_1.motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.1 }} className="glass-card p-6">
          <h3 className="section-title flex items-center gap-2 mb-4">
            <hi2_1.HiOutlineFingerPrint size={20} className="text-primary-400"/>
            Integrity Checks
          </h3>

          <div className="space-y-3">
            {integrityReport.checks.map(function (check, idx) {
                var colors = statusColors[check.status];
                return (<div key={idx} className={"flex items-center justify-between p-4 rounded-xl border ".concat(colors === null || colors === void 0 ? void 0 : colors.border, " ").concat(colors === null || colors === void 0 ? void 0 : colors.bg)}>
                  <div className="flex items-center gap-3">
                    <span className={"text-lg ".concat(colors === null || colors === void 0 ? void 0 : colors.text)}>
                      {check.status === 'Ok' ? '✓' : check.status === 'Warning' ? '⚠' : '✗'}
                    </span>
                    <div>
                      <h4 className="font-medium text-dark-100">{check.component}</h4>
                      <p className="text-xs text-dark-400 mt-0.5 font-mono">{check.details}</p>
                    </div>
                  </div>
                  <span className={"text-sm font-medium ".concat(colors === null || colors === void 0 ? void 0 : colors.text)}>{check.status}</span>
                </div>);
            })}
          </div>
        </framer_motion_1.motion.div>)}

      {/* Cryptographic Capabilities */}
      {securityInfo && (<framer_motion_1.motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.2 }} className="glass-card p-6">
          <h3 className="section-title flex items-center gap-2 mb-4">
            <hi2_1.HiOutlineLockClosed size={20} className="text-quantum-400"/>
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
                {securityInfo.supported_algorithms.map(function (algo) { return (<tr key={algo.name} className="border-b border-dark-800/50 hover:bg-dark-800/30">
                    <td className="py-3 px-4 font-semibold text-dark-100">{algo.name}</td>
                    <td className="py-3 px-4 text-dark-300 font-mono text-xs">{algo.algorithm_type}</td>
                    <td className="py-3 px-4 text-dark-300 font-mono text-xs">{algo.key_size}</td>
                    <td className="py-3 px-4 text-dark-300 font-mono text-xs">{algo.security_level}</td>
                    <td className="py-3 px-4">
                      {algo.quantum_safe ? (<span className="quantum-badge">🛡️ Yes</span>) : (<span className="status-warning">⚠️ No</span>)}
                    </td>
                  </tr>); })}
              </tbody>
            </table>
          </div>

          <div className="mt-4 p-4 bg-dark-800/30 rounded-xl border border-dark-700/30">
            <h4 className="text-sm font-semibold text-dark-300 mb-2 flex items-center gap-2">
              <hi2_1.HiOutlineCpuChip size={16}/>
              Hash Algorithms
            </h4>
            <div className="flex gap-2 flex-wrap">
              {securityInfo.hash_algorithms.map(function (hash) { return (<span key={hash} className="status-info">
                  {hash}
                </span>); })}
            </div>
          </div>
        </framer_motion_1.motion.div>)}

      {/* Threat Model Summary */}
      <framer_motion_1.motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.3 }} className="glass-card p-6">
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
        ].map(function (item, idx) { return (<div key={idx} className="bg-dark-800/50 p-4 rounded-xl border border-dark-700/30">
              <h4 className="font-semibold text-dark-200 text-sm mb-1">{item.threat}</h4>
              <p className="text-xs text-dark-400 mb-2">{item.mitigation}</p>
              <span className={item.status === 'Mitigated' ? 'status-ok' : 'status-warning'}>
                {item.status}
              </span>
            </div>); })}
        </div>
      </framer_motion_1.motion.div>
    </div>);
};
exports.default = SecurityStatus;
