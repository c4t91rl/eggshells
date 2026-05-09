"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var react_1 = require("react");
var Sidebar_1 = require("./Sidebar");
var Layout = function (_a) {
    var children = _a.children;
    return (<div className="flex h-screen w-screen overflow-hidden">
      <Sidebar_1.default />
      <main className="flex-1 overflow-y-auto p-6">
        <div className="max-w-7xl mx-auto animate-fade-in">
          {children}
        </div>
      </main>
    </div>);
};
exports.default = Layout;
