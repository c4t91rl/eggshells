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
var jsx_runtime_1 = require("react/jsx-runtime");
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
    return ((0, jsx_runtime_1.jsxs)("div", { className: "space-y-6", children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-center justify-between", children: [(0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h2", { className: "text-2xl font-bold text-dark-50", children: "Security Status" }), (0, jsx_runtime_1.jsx)("p", { className: "text-dark-400 text-sm mt-1", children: "System integrity, cryptographic capabilities, and threat analysis" })] }), (0, jsx_runtime_1.jsxs)("button", { onClick: handleRefreshIntegrity, className: "btn-secondary flex items-center gap-2", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineArrowPath, { size: 18 }), "Re-check Integrity"] })] }), integrityReport && ((0, jsx_runtime_1.jsx)(framer_motion_1.motion.div, { initial: { opacity: 0, y: 20 }, animate: { opacity: 1, y: 0 }, className: "glass-card p-6 border-2 ".concat((_a = statusColors[integrityReport.overall_status]) === null || _a === void 0 ? void 0 : _a.border), children: (0, jsx_runtime_1.jsxs)("div", { className: "flex items-center gap-4", children: [(0, jsx_runtime_1.jsx)("div", { className: "p-3 rounded-2xl ".concat((_b = statusColors[integrityReport.overall_status]) === null || _b === void 0 ? void 0 : _b.bg), children: integrityReport.overall_status === 'Ok' ? ((0, jsx_runtime_1.jsx)(hi2_1.HiOutlineShieldCheck, { size: 40, className: "text-green-400" })) : ((0, jsx_runtime_1.jsx)(hi2_1.HiOutlineShieldExclamation, { size: 40, className: "text-yellow-400" })) }), (0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsxs)("h3", { className: "text-2xl font-bold ".concat((_c = statusColors[integrityReport.overall_status]) === null || _c === void 0 ? void 0 : _c.text), children: ["System ", integrityReport.overall_status] }), (0, jsx_runtime_1.jsxs)("p", { className: "text-dark-400 text-sm", children: ["Last checked: ", new Date(integrityReport.timestamp).toLocaleString()] })] })] }) })), integrityReport && ((0, jsx_runtime_1.jsxs)(framer_motion_1.motion.div, { initial: { opacity: 0, y: 20 }, animate: { opacity: 1, y: 0 }, transition: { delay: 0.1 }, className: "glass-card p-6", children: [(0, jsx_runtime_1.jsxs)("h3", { className: "section-title flex items-center gap-2 mb-4", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineFingerPrint, { size: 20, className: "text-primary-400" }), "Integrity Checks"] }), (0, jsx_runtime_1.jsx)("div", { className: "space-y-3", children: integrityReport.checks.map(function (check, idx) {
                            var colors = statusColors[check.status];
                            return ((0, jsx_runtime_1.jsxs)("div", { className: "flex items-center justify-between p-4 rounded-xl border ".concat(colors === null || colors === void 0 ? void 0 : colors.border, " ").concat(colors === null || colors === void 0 ? void 0 : colors.bg), children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-center gap-3", children: [(0, jsx_runtime_1.jsx)("span", { className: "text-lg ".concat(colors === null || colors === void 0 ? void 0 : colors.text), children: check.status === 'Ok' ? '✓' : check.status === 'Warning' ? '⚠' : '✗' }), (0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h4", { className: "font-medium text-dark-100", children: check.component }), (0, jsx_runtime_1.jsx)("p", { className: "text-xs text-dark-400 mt-0.5 font-mono", children: check.details })] })] }), (0, jsx_runtime_1.jsx)("span", { className: "text-sm font-medium ".concat(colors === null || colors === void 0 ? void 0 : colors.text), children: check.status })] }, idx));
                        }) })] })), securityInfo && ((0, jsx_runtime_1.jsxs)(framer_motion_1.motion.div, { initial: { opacity: 0, y: 20 }, animate: { opacity: 1, y: 0 }, transition: { delay: 0.2 }, className: "glass-card p-6", children: [(0, jsx_runtime_1.jsxs)("h3", { className: "section-title flex items-center gap-2 mb-4", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineLockClosed, { size: 20, className: "text-quantum-400" }), "Cryptographic Capabilities"] }), (0, jsx_runtime_1.jsx)("div", { className: "overflow-x-auto", children: (0, jsx_runtime_1.jsxs)("table", { className: "w-full text-sm", children: [(0, jsx_runtime_1.jsx)("thead", { children: (0, jsx_runtime_1.jsxs)("tr", { className: "border-b border-dark-700/50", children: [(0, jsx_runtime_1.jsx)("th", { className: "text-left py-3 px-4 text-dark-400 font-medium", children: "Algorithm" }), (0, jsx_runtime_1.jsx)("th", { className: "text-left py-3 px-4 text-dark-400 font-medium", children: "Type" }), (0, jsx_runtime_1.jsx)("th", { className: "text-left py-3 px-4 text-dark-400 font-medium", children: "Key Size" }), (0, jsx_runtime_1.jsx)("th", { className: "text-left py-3 px-4 text-dark-400 font-medium", children: "Security Level" }), (0, jsx_runtime_1.jsx)("th", { className: "text-left py-3 px-4 text-dark-400 font-medium", children: "Quantum Safe" })] }) }), (0, jsx_runtime_1.jsx)("tbody", { children: securityInfo.supported_algorithms.map(function (algo) { return ((0, jsx_runtime_1.jsxs)("tr", { className: "border-b border-dark-800/50 hover:bg-dark-800/30", children: [(0, jsx_runtime_1.jsx)("td", { className: "py-3 px-4 font-semibold text-dark-100", children: algo.name }), (0, jsx_runtime_1.jsx)("td", { className: "py-3 px-4 text-dark-300 font-mono text-xs", children: algo.algorithm_type }), (0, jsx_runtime_1.jsx)("td", { className: "py-3 px-4 text-dark-300 font-mono text-xs", children: algo.key_size }), (0, jsx_runtime_1.jsx)("td", { className: "py-3 px-4 text-dark-300 font-mono text-xs", children: algo.security_level }), (0, jsx_runtime_1.jsx)("td", { className: "py-3 px-4", children: algo.quantum_safe ? ((0, jsx_runtime_1.jsx)("span", { className: "quantum-badge", children: "\uD83D\uDEE1\uFE0F Yes" })) : ((0, jsx_runtime_1.jsx)("span", { className: "status-warning", children: "\u26A0\uFE0F No" })) })] }, algo.name)); }) })] }) }), (0, jsx_runtime_1.jsxs)("div", { className: "mt-4 p-4 bg-dark-800/30 rounded-xl border border-dark-700/30", children: [(0, jsx_runtime_1.jsxs)("h4", { className: "text-sm font-semibold text-dark-300 mb-2 flex items-center gap-2", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineCpuChip, { size: 16 }), "Hash Algorithms"] }), (0, jsx_runtime_1.jsx)("div", { className: "flex gap-2 flex-wrap", children: securityInfo.hash_algorithms.map(function (hash) { return ((0, jsx_runtime_1.jsx)("span", { className: "status-info", children: hash }, hash)); }) })] })] })), (0, jsx_runtime_1.jsxs)(framer_motion_1.motion.div, { initial: { opacity: 0, y: 20 }, animate: { opacity: 1, y: 0 }, transition: { delay: 0.3 }, className: "glass-card p-6", children: [(0, jsx_runtime_1.jsx)("h3", { className: "section-title mb-4", children: "Threat Model" }), (0, jsx_runtime_1.jsx)("div", { className: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4", children: [
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
                        ].map(function (item, idx) { return ((0, jsx_runtime_1.jsxs)("div", { className: "bg-dark-800/50 p-4 rounded-xl border border-dark-700/30", children: [(0, jsx_runtime_1.jsx)("h4", { className: "font-semibold text-dark-200 text-sm mb-1", children: item.threat }), (0, jsx_runtime_1.jsx)("p", { className: "text-xs text-dark-400 mb-2", children: item.mitigation }), (0, jsx_runtime_1.jsx)("span", { className: item.status === 'Mitigated' ? 'status-ok' : 'status-warning', children: item.status })] }, idx)); }) })] })] }));
};
exports.default = SecurityStatus;
