"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var jsx_runtime_1 = require("react/jsx-runtime");
var VerificationBadge = function (_a) {
    var algorithm = _a.algorithm, _b = _a.className, className = _b === void 0 ? '' : _b;
    var getConfig = function () {
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
    var config = getConfig();
    return ((0, jsx_runtime_1.jsxs)("span", { className: "".concat(config.className, " ").concat(className), title: config.tooltip, children: [config.icon, " ", config.label] }));
};
exports.default = VerificationBadge;
