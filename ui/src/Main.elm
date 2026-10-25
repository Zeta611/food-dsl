module Main exposing (main)

import Browser
import Html exposing (Html, button, div, h1, node, span, text)
import Html.Attributes exposing (attribute, class)
import Html.Events exposing (onClick)


main : Program Int Model Msg
main =
    Browser.element { init = init, view = view, update = update, subscriptions = subscriptions }


type alias Model =
    { cnt : Int }


init : Int -> ( Model, Cmd Msg )
init cnt =
    ( { cnt = cnt }, Cmd.none )


type Msg
    = Increment
    | Decrement
    | Reset


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Increment ->
            ( { model | cnt = model.cnt + 1 }
            , Cmd.none
            )

        Decrement ->
            ( { model | cnt = model.cnt - 1 }
            , Cmd.none
            )

        Reset ->
            ( { model | cnt = 0 }
            , Cmd.none
            )


view : Model -> Html Msg
view model =
    div [ class "p-3" ]
        [ h1 [ class "text-3xl font-bold" ] [ text "Hello, World!" ]
        , div [ class "grid min-h-screen place-items-center" ]
            [ div []
                [ node
                    "render-view"
                    [ attribute "value" (String.fromInt model.cnt)
                    ]
                    []
                , span [ class "flex gap-3 place-items-center" ]
                    [ button [ onClick Decrement, class "btn" ] [ text "-" ]
                    , div [] [ text (String.fromInt model.cnt) ]
                    , button [ onClick Increment, class "btn" ] [ text "+" ]
                    , button [ onClick Reset, class "btn" ] [ text "🔥" ]
                    ]
                ]
            ]
        ]


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none
