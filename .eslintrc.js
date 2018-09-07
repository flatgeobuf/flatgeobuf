module.exports = {
  root: true,
  parserOptions: {
    ecmaVersion: 2017,
    sourceType: 'module'
  },
  plugins: [
    'mocha'
  ],
  extends: [
    'eslint:recommended'
  ],
  env: {
    browser: true,
    node: true
  },
  rules: {
    semi: ['error', 'never']
  },
  overrides: [
    {
      files: ["test/**/*.js"],
      env: { "mocha": true }
    }
  ]
}