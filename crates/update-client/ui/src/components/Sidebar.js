"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var jsx_runtime_1 = require("react/jsx-runtime");
var appStore_1 = require("../store/appStore");
var hi2_1 = require("react-icons/hi2");
var io5_1 = require("react-icons/io5");
var Sidebar = function () {
    var _a;
    var _b = (0, appStore_1.useAppStore)(), currentPage = _b.currentPage, setPage = _b.setPage, availableUpdates = _b.availableUpdates, integrityReport = _b.integrityReport;
    var menuItems = [
        { id: 'dashboard', label: 'Dashboard', icon: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineHome, { size: 20 }) },
        { id: 'servers', label: 'Servers', icon: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineServer, { size: 20 }) },
        {
            id: 'updates',
            label: 'Updates',
            icon: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineArrowDown, { size: 20 }),
            badge: availableUpdates.length || undefined,
        },
        { id: 'security', label: 'Security', icon: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineShieldCheck, { size: 20 }) },
        { id: 'settings', label: 'Settings', icon: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineCog6Tooth, { size: 20 }) },
        { id: 'logs', label: 'Activity Log', icon: (0, jsx_runtime_1.jsx)(hi2_1.HiOutlineDocumentText, { size: 20 }) },
    ];
    return ((0, jsx_runtime_1.jsxs)("aside", { className: "w-64 h-screen bg-dark-900/80 backdrop-blur-xl border-r border-dark-700/50 flex flex-col", children: [(0, jsx_runtime_1.jsxs)("div", { className: "p-6 flex items-center gap-3", children: [(0, jsx_runtime_1.jsxs)("div", { className: "relative", children: [(0, jsx_runtime_1.jsx)(io5_1.IoShieldCheckmark, { size: 32, className: "text-quantum-400 animate-shield-glow" }), (0, jsx_runtime_1.jsx)("div", { className: "absolute -top-1 -right-1 w-3 h-3 bg-green-400 rounded-full border-2 border-dark-900" })] }), (0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h1", { className: "text-lg font-bold gradient-text", children: "KryptoUpdate" }), (0, jsx_runtime_1.jsx)("p", { className: "text-xs text-dark-500", children: "Post-Quantum Secure" })] })] }), (0, jsx_runtime_1.jsx)("nav", { className: "flex-1 px-3 space-y-1", children: menuItems.map(function (item) { return ((0, jsx_runtime_1.jsxs)("button", { onClick: function () { return setPage(item.id); }, className: "w-full ".concat(currentPage === item.id ? 'sidebar-item-active' : 'sidebar-item'), children: [item.icon, (0, jsx_runtime_1.jsx)("span", { className: "flex-1 text-left text-sm font-medium", children: item.label }), item.badge && ((0, jsx_runtime_1.jsx)("span", { className: "bg-primary-600 text-white text-xs px-2 py-0.5 rounded-full", children: item.badge }))] }, item.id)); }) }), (0, jsx_runtime_1.jsx)("div", { className: "p-4 border-t border-dark-700/50", children: (0, jsx_runtime_1.jsxs)("div", { className: "glass-card p-3 rounded-xl", children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-center gap-2 mb-2", children: [(0, jsx_runtime_1.jsx)("div", { className: "w-2 h-2 rounded-full ".concat((integrityReport === null || integrityReport === void 0 ? void 0 : integrityReport.overall_status) === 'Ok'
                                        ? 'bg-green-400'
                                        : (integrityReport === null || integrityReport === void 0 ? void 0 : integrityReport.overall_status) === 'Warning'
                                            ? 'bg-yellow-400'
                                            : 'bg-red-400', " animate-pulse") }), (0, jsx_runtime_1.jsx)("span", { className: "text-xs text-dark-300", children: "System Integrity" })] }), (0, jsx_runtime_1.jsx)("p", { className: "text-xs text-dark-500 font-mono", children: (_a = integrityReport === null || integrityReport === void 0 ? void 0 : integrityReport.overall_status) !== null && _a !== void 0 ? _a : 'Checking...' })] }) })] }));
};
exports.default = Sidebar;
