export default {
  compilerOptions: {
    css: "injected",
    warningFilter: (warning) => {
      if (warning.code && (warning.code.startsWith("a11y_") || warning.code === "state_referenced_locally")) {
        return false;
      }
      return true;
    },
  },
};
