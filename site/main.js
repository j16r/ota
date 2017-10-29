var IntroScreen = React.createClass({
  handleError: function(err) {
    console.error(err)
  },
  render: function() {
    return (
      <div>
        <section>
          <div>
            <h1>
              ota
            </h1>
          </div>
          <div>
            <h2>
              click anywhere to create
            </h2>
          </div>
        </section>
      </div>
    )
  }
})

ReactDOM.render(
  <IntroScreen></IntroScreen>,
  document.getElementById('main')
);
