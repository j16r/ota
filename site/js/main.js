console.log("loading main.js");

class IntroScreen extends React.Component {
  handleError(err) {
    console.error(err);
  }

  onClick() {
    console.error("clicked");
  }

  render() {
    return (
      <div onClick={this.onClick}>
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
}

const introScreen = <IntroScreen/>;
ReactDOM.render(
  introScreen,
  document.getElementById('root')
);
